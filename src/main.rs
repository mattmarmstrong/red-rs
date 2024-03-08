use std::io::{Read, Write};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::io;

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

fn main() {
    static PING_COUNT: AtomicUsize = AtomicUsize::new(0);
    const TOKEN: Token = Token(0);
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(128);
    let mut listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 6379))).unwrap();
    poll.registry()
        .register(&mut listener, TOKEN, Interest::READABLE)
        .unwrap();
    'outer: loop {
        poll.poll(&mut events, Some(Duration::from_millis(1000)))
            .unwrap();
        for event in events.iter() {
            match event.token() {
                TOKEN => 'inner: loop {
                    match listener.accept() {
                        Ok((mut stream, connection)) => {
                            println!("Accepted connection from: {}", connection);
                            handle_connection(&mut stream).unwrap();
                            PING_COUNT.fetch_add(1, Ordering::SeqCst);
                            if PING_COUNT.load(Ordering::SeqCst) == 2 {
                                break 'outer;
                            }
                        }
                        Err(ref err) if would_block(err) => break 'outer,
                        Err(e) => eprintln!("{}", e),
                    }
                },
                _ => panic!(),
            }
        }
    }
}

fn handle_connection(mut stream: &TcpStream) -> anyhow::Result<()> {
    const PONG: &str = "+PONG\r\n";
    let mut buffer = [0; 1024];
    while let Ok(val) = stream.read(&mut buffer) {
        if val != 0 {
            stream.write_all(PONG.as_bytes()).unwrap();
            stream.flush()?
        }
    }

    Ok(())
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}
