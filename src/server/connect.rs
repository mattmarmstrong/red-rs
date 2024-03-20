use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::resp::data::DataType;
use crate::resp::parse::Parser;

type R<T> = anyhow::Result<T>;

pub async fn read_exact(_stream: &mut TcpStream) -> R<Option<DataType>> {
    todo!()
}

pub async fn read(stream: &mut TcpStream) -> R<Option<DataType>> {
    let mut buffer = [0u8; 1024];
    loop {
        let bytes_read = stream.read(&mut buffer).await.expect("Read failed!");
        if bytes_read != 0 {
            break;
        }
    }

    let parsed_data = Parser::new(&buffer).parse().ok();
    Ok(parsed_data)
}

pub async fn write(stream: &mut TcpStream, msg: String) -> R<()> {
    stream
        .write_all(msg.as_bytes())
        .await
        .expect("Write failed!");
    stream.flush().await.expect("Flush failed!");
    Ok(())
}
