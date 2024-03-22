use tokio::net::TcpStream;

pub mod command;
pub mod errors;
pub mod info;

#[derive(Debug)]
pub struct Replica {
    pub port: u16,
    pub stream: TcpStream,
}

impl Replica {
    pub fn new(port: u16, stream: TcpStream) -> Self {
        Self { port, stream }
    }
}
