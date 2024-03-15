use std::io::Write;
use std::net::TcpStream;

use crate::resp::parse::Parser;
use crate::resp::serialize::Serializer;
use crate::server::read_bytes_sync;
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

fn do_slave_ping(stream: &mut TcpStream) -> R<()> {
    let ping = Serializer::to_arr(Vec::from(["ping"]));
    stream.write_all(ping.as_bytes()).expect("Failed to write!");
    let ping_resp = read_bytes_sync(stream);
    expected_response("ping", &ping_resp)
}

fn do_slave_listen(stream: &mut TcpStream, server: &Server) -> R<()> {
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

fn do_slave_psync(stream: &mut TcpStream) -> R<()> {
    let psync = Serializer::to_arr(Vec::from(["REPLCONF", "capa", "psync2"]));
    stream
        .write_all(psync.as_bytes())
        .expect("Failed to write!");
    let listen_resp = read_bytes_sync(stream);
    expected_response("ok", &listen_resp)
}

pub fn do_slave_handshake(server: &Server) -> R<()> {
    // Another mess. FIXME
    // TODO -> The rest of the repl_handshake
    debug_assert!(server.replica_info.role == Role::Slave);
    debug_assert!(server.master_addr().is_some());
    match TcpStream::connect(server.master_addr().unwrap()) {
        Ok(mut master_stream) => {
            do_slave_ping(&mut master_stream)?;
            do_slave_listen(&mut master_stream, &server)?;
            do_slave_psync(&mut master_stream)?;
            Ok(())
        }
        Err(_) => Err(ReplError::FailedToConnect),
    }
}
