use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    const PONG: &'static str = "+PONG\r\n";
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                while let Ok(val) = stream.read(&mut buffer) {
                    if val != 0 {
                        stream.write(PONG.as_bytes()).unwrap();
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
