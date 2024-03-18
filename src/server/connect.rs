use bytes::BytesMut;
use std::io::{Read, Write};
use std::net::TcpStream;

type R<T> = anyhow::Result<T>;

pub struct Connection {
    pub stream: TcpStream,
    pub buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(4096),
        }
    }

    #[inline]
    pub fn buf_clear(&mut self) -> R<()> {
        self.buffer.clear();
        Ok(())
    }

    pub fn read(&mut self) -> R<()> {
        let mut stream = self.stream.try_clone().unwrap();
        loop {
            let bytes_read = stream.read(&mut self.buffer).expect("Read failed!");
            if bytes_read == 0 {
                break;
            }
        }

        Ok(())
    }

    pub fn write(&mut self, msg: String) -> R<()> {
        println!("WRITING! MSG: {}", msg);
        self.stream.write(msg.as_bytes()).expect("Write failed!");
        self.stream.flush().expect("Flush failed!");
        Ok(())
    }
}
