use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub mod command;
pub mod errors;
pub mod info;

#[derive(Debug)]
pub struct Replica {
    pub socket_addr: SocketAddr,
    pub stream: Arc<Mutex<TcpStream>>,
}

impl Replica {
    pub fn new(socket_addr: SocketAddr, stream: Arc<Mutex<TcpStream>>) -> Self {
        Self {
            socket_addr,
            stream,
        }
    }
}
