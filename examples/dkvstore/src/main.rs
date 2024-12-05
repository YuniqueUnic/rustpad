use std::time::Duration;

use futures::StreamExt;
// use anyhow::{Ok, Result};
use libp2p::{
    kad::{self, store::MemoryStore},
    mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, SwarmBuilder,
};
use tokio::{
    self,
    io::{self, AsyncBufReadExt},
    select,
};
use tracing_subscriber::EnvFilter;

// NetworkBehaviour 派生宏：自动实现网络行为的委托和集成
// 允许组合多个网络协议行为（Kademlia DHT + mDNS 服务发现）
#[derive(NetworkBehaviour)]
struct Behavior {
    kademlia: kad::Behaviour<MemoryStore>, // Kademlia 分布式哈希表：用于节点路由和数据存储
    mdns: mdns::tokio::Behaviour,          // mDNS 本地服务发现：在局域网内自动发现对等节点
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    // SwarmBuilder: 构建点对点网络的核心组件
    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio() // 使用 Tokio 运行时进行异步网络操作
        .with_tcp(
            tcp::Config::default(), // TCP 传输层配置：提供基础网络连接 |处理套接字创建和管理 | 支持 IPv4/IPv6
            noise::Config::new,     // 安全传输层：Noise 协议，提供加密、身份验证和前向保密
            yamux::Config::default, //多路复用协议：Yamux，在单个 TCP 连接上复用多个子流，提高网络效率和并发性
        )?
        .with_behaviour(|key| {
            // 网络行为配置
            Ok(Behavior {
                //Kademlia 分布式哈希表初始化：节点路由 | 去中心化数据存储 | 点对点服务发现
                kademlia: kad::Behaviour::new(
                    key.public().to_peer_id(),                   // 使用公钥生成唯一 PeerID
                    MemoryStore::new(key.public().to_peer_id()), // 内存存储 DHT 数据
                ),
                // mDNS 本地服务发现：自动发现同一局域网内的节点
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?,
            })
        })?
        // with_swarm_config configuration: <details: What? When? Why?>
        .with_swarm_config(
            |c| 
            // 空闲连接超时：释放未使用资源
            c.with_idle_connection_timeout(Duration::from_secs(60)), // 连接池大小控制等等..
                                                                     // .with_max_negotiating_inbound_streams(10)
        )
        .build();

    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server)); // <details: What? When? Why?>

    // read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // listen on all interfaces and whatever port the OS assigns.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // kick it off
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line()=>{
                handle_input_line(&mut swarm.behaviour_mut().kademlia, line);
            }
            event = swarm.select_next_some() => match event{
                SwarmEvent::NewListenAddr{address,..}=>{
                    println!("Listening on {:?}", address);
                },
                SwarmEvent::Behaviour(BehaviorEvent::Mdns(mdns::Event::Discovered(list)))=>{
                   for (peer_id,multiaddr) in list {
                        swarm.behaviour_mut().kademlia.add_address(&peer_id,multiaddr);
                   }
                },
                // todo: handle other events
                _=>{
                    println!("None handler for event: {:30?}",event);
                }
            }
        }
    }

    Ok(())
}

const METHODS: [&'static str; 4] = ["GET", "PUT", "GET_PROVIDERS", "PUT_PROVIDER"];
fn handle_input_line(kademlia: &mut kad::Behaviour<MemoryStore>, line: String) {
    let mut args = line.split_ascii_whitespace();

    match args.next() {
        Some("GET") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprint!("Expected a key");
                        return;
                    }
                }
            };
        }
        Some("GET_PROVIDERS") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key), // <details: What? When? Why?>
                    None => {
                        eprint!("Expected a key");
                        return;
                    }
                }
            };
            kademlia.get_providers(key);
        }
        Some("PUT") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprint!("missing key");
                        return;
                    }
                }
            };
            let value:Vec<u8> ={
                match args.next() {
                    Some(v)=> v.as_bytes().to_vec(), // <details: What? When? Why?>
                    None=>{
                        eprint!("Expected value");
                        return;
                    }
                }
            };
            let record = kad::Record{
                key,
                value,
                publisher:None,
                expires:None
            };
            kademlia.put_record(record, kad::Quorum::One) // <details: What? When? Why?>
                    .expect("Failed to put record");
        }
        Some("PUT_PROVIDER") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprint!("missing key");
                        return;
                    }
                }
            };

            kademlia.start_providing(key) // <details: What? When? Why?>
                    .expect("Failed to start providing");
        }
        _ => {
            eprintln!("unknown command");
            eprintln!("Expecting one of: {:?}",METHODS);
        }
    }
}
