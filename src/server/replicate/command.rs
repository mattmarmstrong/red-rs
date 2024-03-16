use tokio::net::TcpStream;

use crate::resp::parse::Parser;

use crate::resp::serialize::Serializer;
use crate::server::connection::Connection;
use crate::server::Server;

use super::errors::ReplError;

type R<T> = anyhow::Result<T, ReplError>;

fn expected_response(expected: &str, actual: &[u8]) -> R<()> {
    if let Ok(resp) = Parser::new(actual).parse() {
        match resp.cmp_str(expected) {
            true => Ok(()),
            false => Err(ReplError::UnexpectedResponse),
        }
    } else {
        Err(ReplError::InvalidResponse)
    }
}

async fn do_follower_ping(c: &mut Connection) -> R<()> {
    let ping = Serializer::to_arr(Vec::from(["ping"]));
    c.write(ping).await.unwrap();
    let ping_resp = c.read().await.expect("Read failed!");
    expected_response("ping", &ping_resp)
}

async fn do_follower_listen(c: &mut Connection, server: &Server) -> R<()> {
    let listen = Serializer::to_arr(Vec::from([
        "REPLCONF",
        "listening-port",
        &server.port.to_string(),
    ]));
    c.write(listen).await.unwrap();
    let listen_resp = c.read().await.expect("Read failed!");
    expected_response("ok", &listen_resp)
}
async fn do_follower_psync(c: &mut Connection) -> R<()> {
    let psync = Serializer::to_arr(Vec::from(["REPLCONF", "capa", "psync2"]));
    c.write(psync).await.unwrap();
    let listen_resp = c.read().await.expect("Read failed!");
    expected_response("ok", &listen_resp)
}

pub async fn do_repl_handshake(server: &Server) -> R<()> {
    // TODO -> The rest of the repl_handshake
    let stream = TcpStream::connect(server.master_addr().unwrap())
        .await
        .expect("Failed to connect!");
    let mut connect = Connection::new(stream);
    do_follower_ping(&mut connect).await?;
    connect.buf_clear().unwrap();
    do_follower_listen(&mut connect, server).await?;
    connect.buf_clear().unwrap();
    do_follower_psync(&mut connect).await?;
    connect.buf_clear().unwrap();
    Ok(())
}
