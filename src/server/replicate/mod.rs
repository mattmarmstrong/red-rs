// use std::net::SocketAddrV4;
use std::sync::Arc;

use tokio::net::TcpStream;

pub mod command;
pub mod errors;
pub mod info;

#[derive(Debug)]
pub struct Replica {
    // pub socket_addr: SocketAddrV4,
    pub stream: Arc<TcpStream>,
}

impl Replica {
    pub fn new(stream: Arc<TcpStream>) -> Self {
        Self { stream }
    }
}
