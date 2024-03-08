use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use mio::{Events, Poll};

fn main() {
    const PONG: &'static str = "+PONG\r\n";
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(128);
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .unwrap();
        for event in events.iter() {
            loop {
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
                        Err(e) => eprintln!("{}", e),
                    }
                }
            }
        }
    }
}
