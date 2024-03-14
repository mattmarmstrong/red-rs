pub mod command;
pub mod errors;
pub mod replicate;
pub mod store;

use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::resp::parse::Parser;

use command::Command;
use replicate::ReplicaInfo;
use store::Store;

#[derive(Debug, Clone)]
pub struct Server {
    pub port: u16,
    pub store: Store,
    pub replica_info: ReplicaInfo,
}

impl Server {
    pub fn new(port: u16, store: Store, replica_info: ReplicaInfo) -> Self {
        Self {
            port,
            store,
            replica_info,
        }
    }

    pub fn default(port: u16) -> Self {
        Self::new(port, Store::new(), ReplicaInfo::default())
    }

    pub fn replicate(port: u16, _master_port: u16) -> Self {
        let replica = Self::new(port, Store::new(), ReplicaInfo::replica());
        // master.replica_info.connected_slaves += 1;
        replica
    }
}

pub async fn handle_connection(stream: &mut TcpStream, server: Arc<Server>) -> anyhow::Result<()> {
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .expect("Failed to read from client stream!");
        if bytes_read == 0 {
            break;
        }

        let mut parser = Parser::new(&buffer);
        let data = parser.parse()?;
        if let Some(cmd) = Command::new(data) {
            cmd.execute(stream, &server).await?;
        }
    }
    Ok(())
}
