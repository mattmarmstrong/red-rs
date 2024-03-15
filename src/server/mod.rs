pub mod command;
pub mod errors;
pub mod replicate;
pub mod store;

use std::io::Read;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::resp::parse::Parser;

use command::Command;
use replicate::info::ReplicaInfo;
use store::Store;

#[derive(Debug, Clone)]
pub struct Server {
    pub port: u16,
    pub master_ip: Option<Ipv4Addr>,
    pub master_port: Option<u16>,
    pub store: Store,
    pub replica_info: ReplicaInfo,
}

impl Server {
    pub fn new(
        port: u16,
        master_ip: Option<Ipv4Addr>,
        master_port: Option<u16>,
        store: Store,
        replica_info: ReplicaInfo,
    ) -> Self {
        Self {
            port,
            master_ip,
            master_port,
            store,
            replica_info,
        }
    }

    pub fn default(port: u16) -> Self {
        Self::new(port, None, None, Store::new(), ReplicaInfo::default())
    }

    pub fn replica(port: u16, master_ip: Ipv4Addr, master_port: u16) -> Self {
        let replica = Self::new(
            port,
            Some(master_ip),
            Some(master_port),
            Store::new(),
            ReplicaInfo::fake_replica(), // TODO -> real
        );
        // TODO -> replication start-up steps here
        replica
    }

    pub fn master_addr(&self) -> Option<SocketAddrV4> {
        match (self.master_ip, self.master_port) {
            (Some(ip), Some(port)) => Some(SocketAddrV4::new(ip, port)),
            _ => None,
        }
    }
}

pub fn init_on_startup(port: Option<u16>, replica_of: Option<Vec<String>>) -> Arc<Server> {
    const DEFAULT_PORT: u16 = 6379;
    let port = port.unwrap_or(DEFAULT_PORT);
    match replica_of {
        // clap handles the parsing for the command args. We can unwrap repl_info safely, because if the arg
        // format is incorrect, this fn won't be called.
        Some(mut repl_info) => {
            // the unwraps here can panic
            let master_port = repl_info.pop().unwrap().parse::<u16>().unwrap();
            let master_ip = match repl_info.pop().unwrap().to_lowercase().as_str() {
                "localhost" => Ipv4Addr::LOCALHOST,
                x => Ipv4Addr::from_str(x).unwrap(),
            };
            Arc::new(Server::replica(port, master_ip, master_port))
        }
        None => Arc::new(Server::default(port)),
    }
}

pub fn read_bytes_sync(stream: &mut std::net::TcpStream) -> [u8; 1024] {
    let mut buffer = [0u8; 1024];
    loop {
        let bytes_read = stream.read(&mut buffer).expect("Failed to read (sync)!");
        if bytes_read == 0 {
            break;
        }
    }
    buffer
}

pub async fn handle_connection(stream: &mut TcpStream, server: Arc<Server>) -> anyhow::Result<()> {
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .expect("Failed to read (async)!");
        if bytes_read == 0 {
            break;
        }
    }
    let mut parser = Parser::new(&buffer);
    let data = parser.parse()?;
    if let Some(cmd) = Command::new(data) {
        cmd.execute(stream, &server).await?;
    }

    Ok(())
}
