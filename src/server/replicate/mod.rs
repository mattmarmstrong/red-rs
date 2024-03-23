// use std::net::SocketAddrV4;
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub mod command;
pub mod errors;
pub mod info;

#[derive(Debug)]
pub struct Replica {
    // pub socket_addr: SocketAddrV4,
    pub stream: Arc<Mutex<TcpStream>>,
}

impl Replica {
    pub fn new(stream: Arc<Mutex<TcpStream>>) -> Self {
        Self { stream }
    }
}
