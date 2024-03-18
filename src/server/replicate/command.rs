use tokio::net::TcpStream;

use crate::resp::serialize::Serializer;
use crate::server::connect::{read, write};
use crate::server::Server;

use super::errors::ReplError;

type R<T> = anyhow::Result<T, ReplError>;

async fn do_follower_ping(s: &mut TcpStream) -> R<()> {
    let ping = Serializer::to_arr(Vec::from(["ping"]));
    write(s, ping).await.expect("Write failed!");
    let ping_resp = read(s).await.expect("Read failed!");
    assert!(ping_resp.unwrap().cmp_str("pong"));
    Ok(())
}

async fn do_follower_listen(s: &mut TcpStream, server: &Server) -> R<()> {
    let listen = Serializer::to_arr(Vec::from([
        "REPLCONF",
        "listening-port",
        &server.port.to_string(),
    ]));
    write(s, listen).await.expect("Write failed!");
    let listen_resp = read(s).await.expect("Read failed!");
    assert!(listen_resp.unwrap().cmp_str("ok"));
    Ok(())
}
async fn do_follower_capa(s: &mut TcpStream) -> R<()> {
    let capa = Serializer::to_arr(Vec::from(["REPLCONF", "capa", "psync2"]));
    write(s, capa).await.expect("Write failed!");
    let capa_resp = read(s).await.expect("Read failed!");
    assert!(capa_resp.unwrap().cmp_str("ok"));
    Ok(())
}

pub async fn do_repl_handshake(server: &Server) -> R<()> {
    // TODO -> The rest of the repl_handshake
    let mut stream = TcpStream::connect(server.master_addr().unwrap())
        .await
        .expect("Failed to connect!");
    do_follower_ping(&mut stream).await?;
    do_follower_listen(&mut stream, server).await?;
    do_follower_capa(&mut stream).await?;
    do_psync(&mut stream, server).await?;
    Ok(())
}

async fn do_psync(s: &mut TcpStream, _server: &Server) -> R<()> {
    let psync = Serializer::to_arr(Vec::from(["PSYNC", "?", "-1"]));
    write(s, psync).await.expect("Write failed!");
    let _psync_resp = read(s).await.expect("Read failed!");
    Ok(())
}
