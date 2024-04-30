pub mod command;
pub mod errors;
pub mod info;

use std::sync::Arc;

use tokio::net::TcpStream;

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
