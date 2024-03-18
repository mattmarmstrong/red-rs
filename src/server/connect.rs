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
            let bytes_read = self
                .stream
                .read_buf(&mut self.buffer)
                .await
                .expect("Read failed!");
            println!("bytes_read: {}", bytes_read);
            if bytes_read == 0 {
                break;
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
