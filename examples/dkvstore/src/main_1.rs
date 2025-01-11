use futures::StreamExt;
use libp2p::{
    kad::{self, store::MemoryStore},
    mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, StreamProtocol, SwarmBuilder,
};
use std::time::Duration;
use tokio::{
    self,
    io::{self, AsyncBufReadExt},
    select,
};
use tracing::{error, warn};
use tracing_subscriber::EnvFilter;

// Network Behaviour: combining Kademlia DHT and mDNS
#[derive(NetworkBehaviour)]
struct KadMdnsBehaviour {
    kademlia: kad::Behaviour<MemoryStore>,
    mdns: mdns::tokio::Behaviour,
}

impl KadMdnsBehaviour {
    fn new(key: &libp2p::identity::Keypair) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(KadMdnsBehaviour {
            kademlia: kad::Behaviour::new(
                key.public().to_peer_id(),
                MemoryStore::new(key.public().to_peer_id()),
            ),
            mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?,
        })
    }
}

// Main function, entry point for async execution
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let key = libp2p::identity::Keypair::generate_ed25519();

    warn!("Public key: ({})", key.public().to_peer_id());

    let mut swarm = build_swarm(&key)?;

    // Start listening on random port
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                handle_input_line(&mut swarm.behaviour_mut().kademlia, line);
            }
            event = swarm.select_next_some() => {
                handle_swarm_event(event, &mut swarm);
            }
        }
    }
}

fn build_swarm(
    key: &libp2p::identity::Keypair,
) -> Result<libp2p::Swarm<KadMdnsBehaviour>, Box<dyn std::error::Error>> {
    let behaviour = KadMdnsBehaviour::new(&key.clone())?;
    let mut swarm = SwarmBuilder::with_existing_identity(key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(move |_| Ok(behaviour))?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    Ok(swarm)
}

// Event handler for the swarm
fn handle_swarm_event(
    event: SwarmEvent<KadMdnsBehaviourEvent>,
    swarm: &mut libp2p::Swarm<KadMdnsBehaviour>,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            warn!("🦀 Listening on {:?}", address);
        }
        SwarmEvent::Behaviour(KadMdnsBehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
            for (peer_id, addr) in peers {
                warn!("📡 Discovered peer {} at {}", peer_id, addr);
                swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr.clone());
                // 立即尝试连接发现的节点
                if let Err(e) = swarm.dial(addr.clone()) {
                    warn!("Failed to dial address {}: {}", addr, e);
                }
            }
        }
        SwarmEvent::ConnectionEstablished {
            peer_id, endpoint, ..
        } => {
            warn!("📲 Connection established with peer: {}", peer_id);
            // 添加连接的节点地址到 Kademlia
            let addr = endpoint.get_remote_address();
            swarm
                .behaviour_mut()
                .kademlia
                .add_address(&peer_id, addr.clone());

            // 尝试引导 Kademlia DHT
            match swarm.behaviour_mut().kademlia.bootstrap() {
                Ok(_) => warn!("🚀 Successfully bootstrapped Kademlia DHT"),
                Err(e) => error!("❌ Failed to bootstrap Kademlia DHT: {}", e),
            }
        }
        SwarmEvent::Behaviour(KadMdnsBehaviourEvent::Kademlia(
            kad::Event::OutboundQueryProgressed { result, .. },
        )) => {
            handle_kad_query_result(result);
        }
        _ => {}
    }
}

fn handle_kad_query_result(result: kad::QueryResult) {
    match result {
        kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
            key,
            providers,
        })) => {
            for peer in providers {
                warn!(
                    "Found provider {} for key {}",
                    peer,
                    String::from_utf8_lossy(key.as_ref())
                );
            }
        }
        kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(kad::PeerRecord {
            peer,
            record: kad::Record { key, value, .. },
        }))) => {
            warn!(
                "Found record for key {}: {}",
                String::from_utf8_lossy(key.as_ref()),
                String::from_utf8_lossy(&value)
            );
        }
        kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
            warn!(
                "Successfully put record with key {}",
                String::from_utf8_lossy(key.as_ref())
            );
        }
        kad::QueryResult::StartProviding(Ok(kad::AddProviderOk { key })) => {
            warn!(
                "Started providing key {}",
                String::from_utf8_lossy(key.as_ref())
            );
        }
        _ => {}
    }
}

fn handle_input_line(kademlia: &mut kad::Behaviour<MemoryStore>, line: String) {
    let mut args = line.split_ascii_whitespace();
    match args.next() {
        Some("GET") => handle_get(kademlia, args),
        Some("PUT") => handle_put(kademlia, args),
        Some("GET_PROVIDERS") => handle_get_providers(kademlia, args),
        Some("PUT_PROVIDER") => handle_put_provider(kademlia, args),
        _ => error!("Unknown command, expected one of: GET, PUT, GET_PROVIDERS, PUT_PROVIDER"),
    }
}

fn handle_get(
    kademlia: &mut kad::Behaviour<MemoryStore>,
    mut args: std::str::SplitAsciiWhitespace,
) {
    if let Some(key) = args.next() {
        kademlia.get_record(kad::RecordKey::new(&key));
    } else {
        error!("Missing key for GET");
    }
}

fn handle_put(
    kademlia: &mut kad::Behaviour<MemoryStore>,
    mut args: std::str::SplitAsciiWhitespace,
) {
    if let Some(key) = args.next() {
        if let Some(value) = args.next() {
            let record = kad::Record {
                key: kad::RecordKey::new(&key),
                value: value.as_bytes().to_vec(),
                publisher: None,
                expires: None,
            };
            kademlia
                .put_record(record, kad::Quorum::One)
                .expect("Failed to put record");
        } else {
            error!("Missing value for PUT");
        }
    } else {
        error!("Missing key for PUT");
    }
}

fn handle_get_providers(
    kademlia: &mut kad::Behaviour<MemoryStore>,
    mut args: std::str::SplitAsciiWhitespace,
) {
    if let Some(key) = args.next() {
        kademlia.get_providers(kad::RecordKey::new(&key));
    } else {
        error!("Missing key for GET_PROVIDERS");
    }
}

fn handle_put_provider(
    kademlia: &mut kad::Behaviour<MemoryStore>,
    mut args: std::str::SplitAsciiWhitespace,
) {
    if let Some(key) = args.next() {
        kademlia
            .start_providing(kad::RecordKey::new(&key))
            .expect("Failed to start providing");
    } else {
        error!("Missing key for PUT_PROVIDER");
    }
}
