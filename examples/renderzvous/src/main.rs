use std::{default, time::Duration};

use futures::StreamExt;
use libp2p::{
    identify,
    identity::{self, Keypair},
    ping, rendezvous,
    swarm::NetworkBehaviour,
};
use tracing_subscriber::EnvFilter;

#[derive(NetworkBehaviour)]
struct RenderzvousServerBehaviour {
    id: identify::Behaviour,
    rendezvous: rendezvous::server::Behaviour,
    ping: ping::Behaviour,
}

impl RenderzvousServerBehaviour {
    fn new(key: &Keypair) -> Self {
        Self {
            id: identify::Behaviour::new(identify::Config::new(
                "renderzvous-exp/1.0.0".to_string(),
                key.public(),
            )),
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::default())
        .try_init();

    let key_pair = libp2p::identity::Keypair::ed25519_from_bytes([0; 32]).unwrap();

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

    let _ = swarm.listen_on("/ip4/0.0.0.0/tcp/64337".parse().unwrap());

    while let Some(event) = swarm.next().await {}

    Ok(())
}
