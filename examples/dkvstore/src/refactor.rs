use futures::StreamExt;
use libp2p::{
    kad::{self, store::MemoryStore, GetProvidersOk, GetRecordOk, PeerRecord, PutRecordOk, Record},
    mdns,
    swarm::{NetworkBehaviour, SwarmEvent},
    SwarmBuilder,
};
use std::{str, time::Duration};

use tokio::io::{self, AsyncBufReadExt};

pub const METHODS: [&str; 4] = ["GET", "PUT", "GET_PROVIDERS", "PUT_PROVIDER"];

// Define the network behavior combining Kademlia DHT and mDNS service discovery
#[derive(NetworkBehaviour)]
pub struct Behavior {
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub mdns: mdns::tokio::Behaviour,
}

// DKVStore structure to encapsulate the DHT functionality
pub struct DKVStore {
    swarm: libp2p::Swarm<Behavior>,
}

impl DKVStore {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut swarm = SwarmBuilder::with_new_identity()
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_behaviour(|key| {
                Ok(Behavior {
                    kademlia: kad::Behaviour::new(
                        key.public().to_peer_id(),
                        MemoryStore::new(key.public().to_peer_id()),
                    ),
                    mdns: mdns::tokio::Behaviour::new(
                        mdns::Config::default(),
                        key.public().to_peer_id(),
                    )?,
                })
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Set Kademlia to server mode
        swarm
            .behaviour_mut()
            .kademlia
            .set_mode(Some(kad::Mode::Server));

        // Listen on all interfaces
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Self { swarm })
    }

    pub async fn handle_event(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}", address);
            }
            SwarmEvent::Behaviour(BehaviorEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, multiaddr);
                }
            }
            SwarmEvent::Behaviour(BehaviorEvent::Kademlia(
                kad::Event::OutboundQueryProgressed { result, .. },
            )) => {
                self.handle_kademlia_query_result(result);
            }
            _ => {
                // Other events can be handled here if needed
            }
        }
        Ok(())
    }

    fn handle_kademlia_query_result(&self, result: kad::QueryResult) {
        match result {
            kad::QueryResult::GetProviders(Ok(GetProvidersOk::FoundProviders {
                key,
                providers,
            })) => {
                for peer in providers {
                    println!(
                        "Found provider {} for key {}",
                        peer,
                        str::from_utf8(key.as_ref()).unwrap()
                    );
                }
            }
            kad::QueryResult::GetProviders(Ok(
                GetProvidersOk::FinishedWithNoAdditionalRecord { closest_peers },
            )) => {
                for peer in closest_peers {
                    println!("Found closest_peers: ({}) ", peer);
                }
            }
            kad::QueryResult::GetProviders(Err(e)) => {
                eprintln!("GetProviders error: {:?}", e);
            }
            kad::QueryResult::GetRecord(Ok(GetRecordOk::FoundRecord(PeerRecord {
                record: Record { key, value, .. },
                ..
            }))) => {
                println!(
                    "Found record {} for key {}",
                    str::from_utf8(value.as_ref()).unwrap(),
                    str::from_utf8(key.as_ref()).unwrap()
                );
            }
            kad::QueryResult::GetRecord(Ok(GetRecordOk::FinishedWithNoAdditionalRecord {
                cache_candidates,
            })) => {
                for (kb_distance, peer_id) in cache_candidates {
                    println!(
                        "Found cache_candidate: ({}) with kb_distance: ({:?})",
                        peer_id,
                        kb_distance.ilog2()
                    );
                }
            }
            kad::QueryResult::GetRecord(Err(e)) => {
                eprint!("GetRecord error: {:?}", e);
            }
            kad::QueryResult::PutRecord(Ok(PutRecordOk { key })) => {
                println!("Put record {}", str::from_utf8(key.as_ref()).unwrap());
            }
            kad::QueryResult::PutRecord(Err(e)) => {
                eprintln!("Put record error: {:?}", e);
            }
            kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
                println!(
                    "Started providing {}",
                    str::from_utf8(key.as_ref()).unwrap()
                );
            }
            kad::QueryResult::StartProviding(Err(e)) => {
                eprintln!("StartProviding error: {:?}", e);
            }
            _ => {
                println!("None handler for event");
            }
        }
    }

    pub fn handle_input(&mut self, line: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut args = line.split(' ');

        match args.next() {
            Some("GET") => {
                let key = match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return Ok(());
                    }
                };
                self.swarm.behaviour_mut().kademlia.get_record(key);
            }
            Some("PUT") => {
                let key = match args.next() {
                    Some(key) => key,
                    None => {
                        eprintln!("Expected key");
                        return Ok(());
                    }
                };
                let value = match args.next() {
                    Some(value) => value,
                    None => {
                        eprintln!("Expected value");
                        return Ok(());
                    }
                };
                let record = Record {
                    key: kad::RecordKey::new(&key),
                    value: value.as_bytes().to_vec(),
                    publisher: None,
                    expires: None,
                };
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .put_record(record, kad::Quorum::One)?;
            }
            Some("GET_PROVIDERS") => {
                let key = match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return Ok(());
                    }
                };
                self.swarm.behaviour_mut().kademlia.get_providers(key);
            }
            Some("PUT_PROVIDER") => {
                let key = match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprintln!("Expected key");
                        return Ok(());
                    }
                };
                self.swarm.behaviour_mut().kademlia.start_providing(key)?;
            }
            _ => {
                eprintln!("expected GET, PUT, GET_PROVIDERS or PUT_PROVIDER");
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdin = io::BufReader::new(io::stdin()).lines();

        loop {
            tokio::select! {
                line = stdin.next_line() => {
                    let line = line?.expect("stdin closed");
                    self.handle_input(line)?;
                }
                _ = self.handle_event() => {}
            }
        }
    }
}
