use std::net::TcpStream;

use crate::resp::parse::Parser;

use crate::resp::serialize::Serializer;
use crate::server::connect::Connection;
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
pub fn do_repl_handshake(server: &Server) -> R<()> {
    // TODO -> The rest of the repl_handshake
    let stream = TcpStream::connect(server.master_addr().unwrap()).unwrap();
    let mut connect = Connection::new(stream);
    let ping = Serializer::to_arr(Vec::from(["ping"]));
    connect.write(ping).expect("Write failed!");
    connect.read().expect("Read failed!");
    expected_response("ping", &mut connect.buffer)?;
    connect.buf_clear().unwrap();
    let listen = Serializer::to_arr(Vec::from([
        "REPLCONF",
        "listening-port",
        &server.port.to_string(),
    ]));
    connect.write(listen).expect("Write failed!");
    connect.read().expect("Read failed!");
    expected_response("ok", &mut connect.buffer)?;
    connect.buf_clear().unwrap();
    let psync = Serializer::to_arr(Vec::from(["REPLCONF", "capa", "psync2"]));
    connect.write(psync).expect("Write failed!");
    connect.read().expect("Read failed!");
    expected_response("ok", &mut connect.buffer)?;
    connect.buf_clear().unwrap();
    Ok(())
}
