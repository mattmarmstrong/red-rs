use std::io::{Read, Write};
use std::net::TcpStream;

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

fn read_bytes_sync(stream: &mut TcpStream) -> [u8; 1024] {
    let mut buffer = [0u8; 1024];
    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .expect("Failed to read from client stream!");
        if bytes_read == 0 {
            break;
        }
    }
    buffer
}

fn do_follower_ping(stream: &mut TcpStream) -> R<()> {
    let ping = Serializer::to_arr(Vec::from(["ping"]));
    stream.write_all(ping.as_bytes()).expect("Failed to write!");
    let ping_resp = read_bytes_sync(stream);
    expected_response("ping", &ping_resp)
}

fn do_follower_listen(stream: &mut TcpStream, server: &Server) -> R<()> {
    let listen = Serializer::to_arr(Vec::from([
        "REPLCONF",
        "listening-port",
        &server.port.to_string(),
    ]));
    stream
        .write_all(listen.as_bytes())
        .expect("Failed to write!");
    let listen_resp = read_bytes_sync(stream);
    expected_response("ok", &listen_resp)
}
fn do_follower_psync(stream: &mut TcpStream) -> R<()> {
    let psync = Serializer::to_arr(Vec::from(["REPLCONF", "capa", "psync2"]));
    stream
        .write_all(psync.as_bytes())
        .expect("Failed to write!");
    let listen_resp = read_bytes_sync(stream);
    expected_response("ok", &listen_resp)
}

pub fn do_repl_handshake(server: &Server) -> R<()> {
    // TODO -> The rest of the repl_handshake
    debug_assert!(server.replica_info.role == Role::Slave);
    debug_assert!(server.master_addr().is_some());
    match TcpStream::connect(server.master_addr().unwrap()) {
        Ok(mut stream) => {
            do_follower_ping(&mut stream)?;
            // do_follower_listen(&mut stream, &server)?;
            // do_follower_psync(&mut stream)?;
            Ok(())
        }
        Err(_) => Err(ReplError::HandshakeFailed),
    }
}
