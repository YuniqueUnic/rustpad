use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    multiaddr, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use std::time::Duration;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;

#[derive(NetworkBehaviour)]
struct RegisterBehaviour {
    rendezvous: rendezvous::client::Behaviour,
    ping: ping::Behaviour,
}

impl RegisterBehaviour {
    fn new(key: &libp2p::identity::Keypair) -> Self {
        Self {
            rendezvous: rendezvous::client::Behaviour::new(key.clone()),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let rendezvous_server_addr = "/ip4/127.0.0.1/tcp/64337".parse::<multiaddr::Multiaddr>()?;
    let rendezvous_point =
        "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN".parse::<libp2p::PeerId>()?;
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|key| RegisterBehaviour::new(key))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    // In production the external address should be the publicly facing IP address of the rendezvous
    // point. This address is recorded in the registration entry by the rendezvous point.
    let external_addr = "/ip4/127.0.0.1/tcp/0".parse::<multiaddr::Multiaddr>()?;
    swarm.add_external_address(external_addr);

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
            SwarmEvent::ConnectionEstablished { peer_id, .. } if peer_id == rendezvous_point => {
                if let Err(error) = swarm.behaviour_mut().rendezvous.register(
                    rendezvous::Namespace::from_static("rendezvous"),
                    rendezvous_point,
                    None,
                ) {
                    error!("Failed to register with rendezvous point: {}", error);
                    anyhow::bail!("Failed to register with rendezvous point: {}", error);
                }
                info!("Connection established with rendezvous point {}", peer_id);
            }
            SwarmEvent::Behaviour(RegisterBehaviourEvent::Rendezvous(
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
            SwarmEvent::Behaviour(RegisterBehaviourEvent::Rendezvous(
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
                );
            }
            SwarmEvent::Behaviour(RegisterBehaviourEvent::Ping(ping::Event {
                peer,
                result: Ok(rtt),
                ..
            })) if peer != rendezvous_point => {
                info!("Ping to {} is {}ms", peer, rtt.as_millis())
            }
            other => {
                debug!("Unhandled {:?}", other);
            }
        }
    }

    Ok(())
}
