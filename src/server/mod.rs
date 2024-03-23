pub mod command;
pub mod connect;
pub mod errors;
pub mod replicate;
pub mod store;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};

use crate::resp::parse::Parser;

use command::Command;
use replicate::info::ReplicaInfo;
use store::Store;

use self::command::CommandResult;
use self::replicate::Replica;

#[derive(Debug)]
pub struct Server {
    pub port: u16,
    pub master_ip: Option<Ipv4Addr>,
    pub master_port: Option<u16>,
    pub store: Store,
    pub replica_info: ReplicaInfo,
    pub replicas: Option<Vec<Replica>>,
    pub repl_queue: Option<Vec<String>>,
}

impl Server {
    pub fn new(
        port: u16,
        master_ip: Option<Ipv4Addr>,
        master_port: Option<u16>,
        store: Store,
        replica_info: ReplicaInfo,
        replicas: Option<Vec<Replica>>,
        repl_queue: Option<Vec<String>>,
    ) -> Self {
        Self {
            port,
            master_ip,
            master_port,
            store,
            replica_info,
            replicas,
            repl_queue,
        }
    }

    pub fn master(port: u16) -> Self {
        Self::new(
            port,
            None,
            None,
            Store::new(),
            ReplicaInfo::master(),
            None,
            None,
        )
    }

    pub fn replica(port: u16, master_ip: Ipv4Addr, master_port: u16) -> Self {
        Self::new(
            port,
            Some(master_ip),
            Some(master_port),
            Store::new(),
            ReplicaInfo::replica(),
            None,
            None,
        )
    }

    pub fn master_addr(&self) -> Option<SocketAddrV4> {
        match (self.master_ip, self.master_port) {
            (Some(ip), Some(port)) => Some(SocketAddrV4::new(ip, port)),
            _ => None,
        }
    }

    // not how I want it to work
    pub async fn propagate(&mut self) -> anyhow::Result<()> {
        println!("{:#?}", self);
        if self.replicas.is_some() {
            let mut remove = Vec::new();
            for (i, repl) in self.replicas.as_mut().unwrap().iter().enumerate() {
                let mut lock = repl.stream.lock().await;
                println!("Replica: {:#?}", repl);
                if self.repl_queue.is_some() {
                    let cmd_q = self.repl_queue.as_mut().unwrap();
                    for cmd in cmd_q {
                        match lock.write_all(cmd.clone().as_bytes()).await {
                            Ok(_) => {
                                continue;
                            }
                            Err(_) => {
                                eprintln!("Replica write failed!");
                                self.replica_info.connected_slaves -= 1;
                                remove.push(i);
                            }
                        }
                    }
                };
                self.repl_queue = None;
            }
            for i in remove.iter().rev() {
                self.replicas.as_mut().unwrap().swap_remove(*i);
            }
        }
        println!("didnt call this?");
        Ok(())
    }
}

pub fn init_on_startup(port: Option<u16>, replica_of: Option<Vec<String>>) -> Arc<RwLock<Server>> {
    const DEFAULT_PORT: u16 = 6379;
    let port = port.unwrap_or(DEFAULT_PORT);
    match replica_of {
        // clap handles the parsing for the command args. We can unwrap repl_info safely, because if the arg
        // format is incorrect, this function won't be called.
        Some(mut repl_info) => {
            // the unwraps here can panic
            let master_port = repl_info.pop().unwrap().parse::<u16>().unwrap();
            let master_ip = match repl_info.pop().unwrap().to_lowercase().as_str() {
                "localhost" => Ipv4Addr::LOCALHOST,
                x => Ipv4Addr::from_str(x).unwrap(),
            };
            Arc::new(RwLock::new(Server::replica(port, master_ip, master_port)))
        }
        None => Arc::new(RwLock::new(Server::master(port))),
    }
}

pub async fn handle_connection(
    stream: Arc<Mutex<TcpStream>>,
    server: &Arc<RwLock<Server>>,
) -> anyhow::Result<()> {
    let mut buffer = [0; 1024];
    loop {
        let _ = stream
            .lock()
            .await
            .read(&mut buffer)
            .await
            .expect("Failed to read from client stream!");
        let mut parser = Parser::new(&buffer);
        let Ok(data) = parser.parse() else {
            break;
        };
        // writing the replication 11 test down in my note
        let mut lock = stream.lock().await;
        let (_, write_stream) = lock.split();
        let Some(cmd) = Command::new(data) else {
            break;
        };
        let res = cmd.execute(write_stream, server).await;
        if res.is_ok() {
            if res.unwrap() == CommandResult::ReplConf {
                let mut server_lock = server.write().await;

                let repl = Replica::new(Arc::clone(&stream));
                if server_lock.replicas.is_some() {
                    server_lock.replicas.as_mut().unwrap().push(repl);
                    server_lock.replica_info.connected_slaves += 1;
                } else {
                    server_lock.replicas = Some(vec![repl]);
                }
            }
        }
    }
    let _ = server.write().await.propagate().await;
    Ok(())
}
