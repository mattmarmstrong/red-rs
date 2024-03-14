use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;

use clap::{arg, Parser};
use tokio::net::TcpListener;

use redis_starter_rust::server::{handle_connection, Server};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, short)]
    port: Option<u16>,
    #[arg(short, long, num_args = 2, value_names = ["MASTER_HOST", "MASTER_PORT"])]
    replicaof: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    const DEFAULT_PORT: u16 = 6379;
    const DEFAULT_IP: Ipv4Addr = Ipv4Addr::LOCALHOST;

    let args = Args::parse();

    let server: Arc<Server>;
    if let Some(mut repl_info) = args.replicaof {
        let master_port = repl_info.pop().unwrap().parse::<u16>().unwrap();
        let _master_ip = match repl_info.pop().unwrap().to_lowercase().as_str() {
            "localhost" => Ipv4Addr::LOCALHOST,
            s => Ipv4Addr::from_str(s).unwrap(),
        };
        server = Arc::new(Server::fake_replicate(
            args.port.unwrap_or(DEFAULT_PORT),
            master_port,
        ));
    } else {
        server = Arc::new(Server::default(args.port.unwrap_or(DEFAULT_PORT)));
    }

    let socket = SocketAddrV4::new(DEFAULT_IP, server.port);
    let listener = TcpListener::bind(socket)
        .await
        .expect("Failed to bind to socket!");
    loop {
        match listener.accept().await {
            Ok((mut stream, client_connection)) => {
                println!("Received connection from client: {}", client_connection);
                let server = Arc::clone(&server);
                tokio::spawn(async move {
                    handle_connection(&mut stream, server).await.unwrap();
                });
            }
            Err(e) => {
                eprintln!("Error accepting client connection: {}", e);
            }
        }
    }
}
