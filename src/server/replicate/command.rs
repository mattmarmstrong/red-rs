use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::resp::parse::Parser;
use crate::resp::serialize::Serializer;
use crate::server::Server;

use super::errors::ReplError;
use super::info::Role;

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

async fn do_slave_ping(stream: &mut TcpStream) -> R<()> {
    let ping = Serializer::to_arr(Vec::from(["ping"]));
    stream
        .write_all(ping.as_bytes())
        .await
        .expect("Failed to write!");

    let mut buffer = [0u8; 1024];
    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .expect("Failed to read (async)!");
        if bytes_read == 0 {
            break;
        }
    }
    expected_response("ping", &buffer)
}

async fn do_slave_listen(stream: &mut TcpStream, server: &Server) -> R<()> {
    let listen = Serializer::to_arr(Vec::from([
        "REPLCONF",
        "listening-port",
        &server.port.to_string(),
    ]));
    stream
        .write_all(listen.as_bytes())
        .await
        .expect("Failed to write!");
    let mut buffer = [0u8; 1024];
    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .expect("Failed to read (async)!");
        if bytes_read == 0 {
            break;
        }
    }
    expected_response("ok", &buffer)
}

async fn do_slave_psync(stream: &mut TcpStream) -> R<()> {
    let psync = Serializer::to_arr(Vec::from(["REPLCONF", "capa", "psync2"]));
    stream
        .write_all(psync.as_bytes())
        .await
        .expect("Failed to write!");
    let mut buffer = [0u8; 1024];
    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .expect("Failed to read (async)!");
        if bytes_read == 0 {
            break;
        }
    }
    expected_response("ok", &buffer)
}

pub async fn do_slave_handshake(server: &Server) -> R<()> {
    // Another mess. FIXME
    // TODO -> The rest of the repl_handshake
    debug_assert!(server.replica_info.role == Role::Slave);
    debug_assert!(server.master_addr().is_some());
    match TcpStream::connect(server.master_addr().unwrap()).await {
        Ok(mut master_stream) => {
            do_slave_ping(&mut master_stream).await?;
            do_slave_listen(&mut master_stream, &server).await?;
            do_slave_psync(&mut master_stream).await?;
            Ok(())
        }
        Err(_) => Err(ReplError::FailedToConnect),
    }
}
