use std::io::{Read, Write};
use std::net::SocketAddr;
use std::time::Duration;

use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};

fn main() {
    const PONG: &str = "+PONG\r\n";
    const PONG_TOKEN: Token = Token(0);
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(128);
    let mut listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 0], 6379))).unwrap();
    poll.registry()
        .register(&mut listener, PONG_TOKEN, Interest::READABLE)
        .unwrap();
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .unwrap();
        for event in events.iter() {
            match event.token() {
                PONG_TOKEN => loop {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let mut buffer = [0; 1024];
                            while let Ok(val) = stream.read(&mut buffer) {
                                if val != 0 {
                                    stream.write(PONG.as_bytes()).unwrap();
                                }
                            }
                        }
                        Err(e) => eprintln!("{}", e),
                    }
                },
                _ => panic!(),
            }
        }
    }
}
