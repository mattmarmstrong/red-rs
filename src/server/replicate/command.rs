use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::RwLock;

use crate::resp::serialize::Serializer;
use crate::server::connect::{expect_resp, write};
use crate::server::Server;

use super::errors::ReplError;

type R<T> = anyhow::Result<T, ReplError>;

async fn do_follower_ping(s: &mut TcpStream) -> R<()> {
    let ping = Serializer::to_arr(Vec::from(["ping"]));
    write(s, ping).await.expect("Write failed!");
    expect_resp(s, "pong").await.expect("Read Failed!");
    Ok(())
}

async fn do_follower_listen(s: &mut TcpStream, server: &Arc<RwLock<Server>>) -> R<()> {
    let listen = Serializer::to_arr(Vec::from([
        "REPLCONF",
        "listening-port",
        &server.read().await.port.to_string(),
    ]));
    write(s, listen).await.expect("Write failed!");
    expect_resp(s, "ok").await.expect("Read failed!");
    Ok(())
}

async fn do_follower_capa(s: &mut TcpStream) -> R<()> {
    let capa = Serializer::to_arr(Vec::from(["REPLCONF", "capa", "psync2"]));
    write(s, capa).await.expect("Write failed!");
    expect_resp(s, "ok").await.expect("Read failed!");
    Ok(())
}

async fn do_follower_psync(s: &mut TcpStream, _server: &Arc<RwLock<Server>>) -> R<()> {
    let psync = Serializer::to_arr(Vec::from(["PSYNC", "?", "-1"]));
    write(s, psync).await.expect("Write failed!");
    expect_resp(s, "ok").await.expect("Read failed!");
    Ok(())
}

pub async fn do_repl_handshake(server: Arc<RwLock<Server>>) -> R<()> {
    // TODO -> The rest of the repl_handshake
    let mut stream = TcpStream::connect(server.read().await.master_addr().unwrap())
        .await
        .expect("Failed to connect!");
    do_follower_ping(&mut stream).await?;
    do_follower_listen(&mut stream, &server).await?;
    do_follower_capa(&mut stream).await?;
    do_follower_psync(&mut stream, &server).await?;
    // handshake complete!

    Ok(())
}
