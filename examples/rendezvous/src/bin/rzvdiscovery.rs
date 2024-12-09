use std::time::Duration;

use futures::StreamExt;
use libp2p::{
    multiaddr::{self, Protocol},
    noise, ping,
    rendezvous::{self},
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr,
};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

const NAMESPACE: &str = "rendezvous";

#[derive(NetworkBehaviour)]
struct DiscoveryBehaviour {
    rendezvous: rendezvous::client::Behaviour,
    ping: ping::Behaviour,
}

impl DiscoveryBehaviour {
    pub fn new(key: &libp2p::identity::Keypair) -> Self {
        Self {
            rendezvous: rendezvous::client::Behaviour::new(key.clone()),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::default())
        .try_init();

    let rendezvous_server_addr = "/ip4/127.0.0.1/tcp/64337".parse::<multiaddr::Multiaddr>()?;
    let rendezvous_point =
        "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN".parse::<libp2p::PeerId>()?;

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| DiscoveryBehaviour::new(key))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    swarm.dial(rendezvous_server_addr.clone())?;

    let mut discover_tick = tokio::time::interval(Duration::from_secs(30));
    let mut cookie = None;

    loop {
        tokio::select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::ConnectionEstablished { peer_id, .. } if peer_id == rendezvous_point => {
                    info!(
                        "Connected to rendezvous point, discovering nodes in '{}' namespace ...",
                        NAMESPACE
                    );
                    swarm.behaviour_mut().rendezvous.discover(
                        Some(rendezvous::Namespace::from_static(&NAMESPACE)),
                        None,
                        None,
                        rendezvous_point,
                    );
                }
                SwarmEvent::Behaviour(DiscoveryBehaviourEvent::Rendezvous(
                    rendezvous::client::Event::Discovered {
                        registrations,
                        cookie: new_cookie,
                        ..
                    },
                )) => {
                    cookie.replace(new_cookie);
                    for reg in registrations {
                        for addr in reg.record.addresses() {
                            let peer = reg.record.peer_id();
                            info!("Discovered peer {:?} at {:?}", peer, addr);

                            let p2p_suffix = Protocol::P2p(peer);
                            let address_with_p2p =
                                if !addr.ends_with(&Multiaddr::empty().with(p2p_suffix.clone())) {
                                    addr.clone().with(p2p_suffix)
                                } else {
                                    addr.clone()
                                };

                            swarm.dial(address_with_p2p)?;
                        }
                    }
                }
                SwarmEvent::Behaviour(DiscoveryBehaviourEvent::Ping(ping::Event {
                    peer,
                    result: Ok(rtt), // RTT: round-trip time - 往返时间
                    ..
                })) if peer != rendezvous_point => {
                    println!("Ping to {} successful in {:?}ms", peer, rtt.as_millis());
                }
                other => {
                    debug!("unhandled {:?}", other);
                }
            },
            _ = discover_tick.tick(), if cookie.is_some()=>{
                swarm.behaviour_mut().rendezvous.discover(
                    Some(rendezvous::Namespace::new(NAMESPACE.to_string()).unwrap()),
                    cookie.clone(),
                    None,
                    rendezvous_point
                );
            }
        }
    }
}
