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

    pub async fn read(&mut self) -> R<Option<&mut BytesMut>> {
        match self.stream.read(&mut self.buffer).await {
            Ok(n) => {
                if n == 0 {
                    return Ok(None);
                };
                Ok(Some(&mut self.buffer))
            }
            Err(_) => panic!("Fix me!"),
        }
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
