use tokio::net::TcpStream;

use crate::resp::serialize::Serializer;
use crate::server::connect::Connection;
use crate::server::Server;

use super::errors::ReplError;

type R<T> = anyhow::Result<T, ReplError>;

async fn do_follower_ping(c: &mut Connection) -> R<()> {
    let ping = Serializer::to_arr(Vec::from(["ping"]));
    c.write(ping).await.expect("Write failed!");
    let ping_resp = c.read().await.expect("Read failed!");
    assert!(ping_resp.unwrap().cmp_str("pong"));
    Ok(())
}

async fn do_follower_listen(c: &mut Connection, server: &Server) -> R<()> {
    let listen = Serializer::to_arr(Vec::from([
        "REPLCONF",
        "listening-port",
        &server.port.to_string(),
    ]));
    c.write(listen).await.expect("Write failed!");
    let listen_resp = c.read().await.expect("Read failed!");
    assert!(listen_resp.unwrap().cmp_str("ok"));
    Ok(())
}
async fn do_follower_psync(c: &mut Connection) -> R<()> {
    let psync = Serializer::to_arr(Vec::from(["REPLCONF", "capa", "psync2"]));
    c.write(psync).await.expect("Write failed!");
    let psync_resp = c.read().await.expect("Read failed!");
    assert!(psync_resp.unwrap().cmp_str("ok"));
    Ok(())
}

pub async fn do_repl_handshake(server: &Server) -> R<()> {
    // TODO -> The rest of the repl_handshake
    let stream = TcpStream::connect(server.master_addr().unwrap())
        .await
        .expect("Failed to connect!");
    let mut connect = Connection::new(stream);
    do_follower_ping(&mut connect).await?;
    do_follower_listen(&mut connect, server).await?;
    do_follower_psync(&mut connect).await?;
    Ok(())
}
