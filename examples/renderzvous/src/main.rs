use std::{default, time::Duration};

use futures::StreamExt;
use libp2p::{
    identify,
    identity::{self, Keypair},
    ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

const PROTOCOL_VERSION: &'static str = "renderzvous-exp/1.0.0";
const MULTIADDR: &'static str = "/ip4/0.0.0.0/tcp/64337";

#[derive(NetworkBehaviour)]
struct RenderzvousServerBehaviour {
    id: identify::Behaviour,
    renderzvous: rendezvous::server::Behaviour,
    ping: ping::Behaviour,
}

impl RenderzvousServerBehaviour {
    fn new(key: &Keypair) -> Self {
        Self {
            id: identify::Behaviour::new(identify::Config::new(
                PROTOCOL_VERSION.to_string(),
                key.public(),
            )),
            renderzvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting rendezvous server...");
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::default())
        .try_init();

    let key_pair = libp2p::identity::Keypair::ed25519_from_bytes([0; 32]).unwrap();
    info!("Local peer id: {}", key_pair.public().to_peer_id());

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key_pair)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|key| RenderzvousServerBehaviour::new(&key))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    swarm.listen_on(MULTIADDR.parse()?)?;

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from {}", peer_id);
            }
            SwarmEvent::Behaviour(RenderzvousServerBehaviourEvent::Renderzvous(
                rendezvous::server::Event::PeerRegistered { peer, registration },
            )) => {
                info!(
                    "Peer {} registered for namespace '{}'",
                    peer, registration.namespace
                );
            }
            SwarmEvent::Behaviour(RenderzvousServerBehaviourEvent::Renderzvous(
                rendezvous::server::Event::DiscoverServed {
                    enquirer,
                    registrations,
                },
            )) => {
                info!(
                    "Served peer {} with {} registrations",
                    enquirer,
                    registrations.len()
                );
            }
            other => {
                debug!("Unhandled {:?}", other);
            }
        }
    }

    Ok(())
}
