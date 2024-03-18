use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

type R<T> = anyhow::Result<T>;

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
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

    pub async fn read(&mut self) -> R<&mut BytesMut> {
        loop {
            self.stream.readable().await.expect("Stream not readable!");
            match self.stream.try_read_buf(&mut self.buffer) {
                Ok(0) => break,
                Ok(n) => println!("Read {} bytes", n),
                Err(e) => println!("Something very bad happened! {}", e),
            }
        }
        Ok(&mut self.buffer)
    }

    pub async fn write(&mut self, msg: String) -> R<()> {
        println!("WRITING! MSG: {}", msg);
        self.stream
            .write_all(msg.as_bytes())
            .await
            .expect("Write failed!");
        self.stream.flush().await.expect("Flush failed!");
        Ok(())
    }
}
