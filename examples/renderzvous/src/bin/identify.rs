use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    identify, multiaddr, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use std::time::Duration;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

const PROTOCOL_VERSION: &'static str = "rendezvous-0.1.0";
const SERVER_MULTIADDR: &'static str = "/ip4/127.0.0.1/tcp/64337";

#[derive(NetworkBehaviour)]
struct IdentifyBehaviour {
    identify: identify::Behaviour,
    rendezvous: rendezvous::client::Behaviour,
    ping: ping::Behaviour,
}

impl IdentifyBehaviour {
    fn new(key: &libp2p::identity::Keypair) -> Self {
        Self {
            identify: identify::Behaviour::new(identify::Config::new(
                PROTOCOL_VERSION.to_string(),
                key.public(),
            )),
            rendezvous: rendezvous::client::Behaviour::new(key.clone()),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::default())
        .try_init();

    let rendezvous_server_addr = SERVER_MULTIADDR.parse::<multiaddr::Multiaddr>()?;
    let rendezvous_point =
        "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN".parse::<libp2p::PeerId>()?;
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|key| IdentifyBehaviour::new(key))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    swarm.dial(rendezvous_server_addr.clone())?;

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => {
                info!("Listening on {} {}", listener_id, address)
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                cause: Some(error),
                ..
            } if peer_id == rendezvous_point => {
                info!("Connection closed with rendezvous point: {}", error);
            }
            SwarmEvent::Behaviour(IdentifyBehaviourEvent::Identify(
                identify::Event::Received { info, .. },
            )) => {
                // Register our external address. Needs to be done explicitly
                // for this case, as it's a local address.
                swarm.add_external_address(info.observed_addr);
                if let Err(e) = swarm.behaviour_mut().rendezvous.register(
                    rendezvous::Namespace::from_static("redenzvous"),
                    rendezvous_point,
                    None,
                ) {
                    error!("Failed to register: {}", e);
                    anyhow::bail!("Failed to register: {}", e);
                }
            }
            SwarmEvent::Behaviour(IdentifyBehaviourEvent::Rendezvous(
                rendezvous::client::Event::Registered {
                    rendezvous_node,
                    ttl,
                    namespace,
                },
            )) => {
                info!(
                    "Registered for namespace '{}' at rendezvous point {} for the next {} seconds",
                    namespace, rendezvous_node, ttl
                );
            }
            SwarmEvent::Behaviour(IdentifyBehaviourEvent::Rendezvous(
                rendezvous::client::Event::RegisterFailed {
                    rendezvous_node,
                    namespace,
                    error,
                },
            )) => {
                error!(
                    "Failed to register: rendezvous_node={}, namespace={}, error_code={:?}",
                    rendezvous_node, namespace, error
                );
                anyhow::bail!(
                    "Failed to register: rendezvous_node={}, namespace={}, error_code={:?}",
                    rendezvous_node,
                    namespace,
                    error
                )
            }
            SwarmEvent::Behaviour(IdentifyBehaviourEvent::Ping(ping::Event {
                peer,
                result: Ok(rtt),
                ..
            })) => {
                info!("Ping to {} is {}ms", peer, rtt.as_millis())
            }
            other => {
                debug!("Unhandled {:?}", other);
            }
        }
    }

    Ok(())
}
