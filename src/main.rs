use std::io::Write;
use std::net::{TcpListener, TcpStream};

fn handle_ping(mut stream: TcpStream) -> anyhow::Result<()> {
    const PONG: &'static str = "+PONG\r\n";
    stream.write(PONG.as_bytes())?;
    stream.write(PONG.as_bytes())?;
    Ok(())
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_ping(stream).unwrap(),
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
