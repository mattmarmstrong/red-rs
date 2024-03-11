use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use tokio::net::TcpListener;

use redis_starter_rust::server::handle_connection;

#[tokio::main]
async fn main() {
    const PORT: u16 = 6379;
    let socket = SocketAddr::from(([127, 0, 0, 1], PORT));
    let listener = TcpListener::bind(socket)
        .await
        .expect("Failed to bind to socket!");
    let store = Arc::new(RwLock::new(HashMap::new()));
    loop {
        match listener.accept().await {
            Ok((mut stream, client_connection)) => {
                println!("Received connection from client: {}", client_connection);
                let store = store.clone();
                tokio::spawn(async move {
                    handle_connection(&mut stream, store).await.unwrap();
                });
            }
            Err(e) => {
                eprintln!("Error accepting client connection: {}", e);
            }
        }
    }
}
