use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::resp::data::DataType;
use crate::resp::parse::Parser;

type R<T> = anyhow::Result<T>;

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub async fn read(&mut self) -> R<Option<DataType>> {
        let mut buffer = [0u8; 1024];
        loop {
            let bytes_read = self.stream.read(&mut buffer).await.expect("Read failed!");
            println!("Bytes read: {}", bytes_read);
            if bytes_read == 0 {
                break;
            }
        }

        let parsed_data = Parser::new(&mut buffer).parse().ok();
        Ok(parsed_data)
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
