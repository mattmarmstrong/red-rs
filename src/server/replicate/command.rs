use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use crate::resp::parse::Parser;
use crate::resp::serialize::Serializer;
use crate::server::Server;

use super::errors::ReplError;

// Put code to communicate between replicas here
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

async fn read_loop(read_stream: &mut ReadHalf<TcpStream>) -> [u8; 1024] {
    let mut buffer = [0u8; 1024];
    loop {
        let bytes_read = read_stream
            .read(&mut buffer)
            .await
            .expect("Failed to read from client stream!");
        if bytes_read == 0 {
            break;
        }
    }
    buffer
}

async fn do_follower_ping(
    read_stream: &mut ReadHalf<TcpStream>,
    write_stream: &mut WriteHalf<TcpStream>,
) -> R<()> {
    let ping = Serializer::to_arr(Vec::from(["ping"]));
    write_stream
        .write_all(ping.as_bytes())
        .await
        .expect("Failed to write!");
    let ping_resp = read_loop(read_stream).await;
    expected_response("ping", &ping_resp)
}

async fn do_follower_listen(
    read_stream: &mut ReadHalf<TcpStream>,
    write_stream: &mut WriteHalf<TcpStream>,
    server: &Server,
) -> R<()> {
    let listen = Serializer::to_arr(Vec::from([
        "REPLCONF",
        "listening-port",
        &server.port.to_string(),
    ]));
    write_stream
        .write_all(listen.as_bytes())
        .await
        .expect("Failed to write!");
    let listen_resp = read_loop(read_stream).await;
    expected_response("ok", &listen_resp)
}
async fn do_follower_psync(
    read_stream: &mut ReadHalf<TcpStream>,
    write_stream: &mut WriteHalf<TcpStream>,
) -> R<()> {
    let psync = Serializer::to_arr(Vec::from(["REPLCONF", "capa", "psync2"]));
    write_stream
        .write_all(psync.as_bytes())
        .await
        .expect("Failed to write!");
    let listen_resp = read_loop(read_stream).await;
    expected_response("ok", &listen_resp)
}

pub async fn do_repl_handshake(server: &Server) -> R<()> {
    // TODO -> The rest of the repl_handshake
    match TcpStream::connect(server.master_addr().unwrap()).await {
        Ok(stream) => {
            let (mut read_stream, mut write_stream) = tokio::io::split(stream);
            do_follower_ping(&mut read_stream, &mut write_stream).await?;
            do_follower_listen(&mut read_stream, &mut write_stream, &server).await?;
            do_follower_psync(&mut read_stream, &mut write_stream).await?;
            Ok(())
        }
        Err(_) => Err(ReplError::HandshakeFailed),
    }
}
