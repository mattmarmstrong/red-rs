use redis_starter_rust::resp::command::Command;
use redis_starter_rust::resp::parse::Parser;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    const PORT: u16 = 6379;
    let socket = SocketAddr::from(([127, 0, 0, 1], PORT));
    let listener = TcpListener::bind(socket)
        .await
        .expect("Failed to bind to socket!");

    'event: loop {
        match listener.accept().await {
            Ok((stream, client_connection)) => {
                println!("Received connection from client: {}", client_connection);
                tokio::spawn(async move {
                    handle_connection(stream).await.unwrap();
                });
            }
            Err(e) => {
                eprintln!("Error accepting client connection: {}", e);
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
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
        if let Some(cmd) = Command::from_dt(&data) {
            if let Some(resp) = cmd.response() {
                stream.write_all(resp).await?;
            }
        }
    }
    Ok(())
}
