use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use clap::{arg, Parser};
use redis_starter_rust::server::replicate::info::Role;
use tokio::net::TcpListener;

use redis_starter_rust::server::replicate::command::do_repl_handshake;
use redis_starter_rust::server::{handle_connection, init_on_startup, Server};

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
    let args = Args::parse();

    let server: Arc<Server> = init_on_startup(args.port, args.replicaof);

    // Move me
    if server.replica_info.role == Role::Slave {
        let s_clone = Arc::clone(&server);
        tokio::spawn(async move {
            do_repl_handshake(&s_clone).await.unwrap_or(());
        });
    }
    // TODO -> un-hardcode localhost
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, server.port);
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
