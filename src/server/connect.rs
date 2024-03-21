use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::resp::data::DataType;
use crate::resp::parse::Parser;

type R<T> = anyhow::Result<T>;

pub async fn read_exact(_stream: &mut TcpStream) -> R<Option<DataType>> {
    todo!()
}

pub async fn expect_resp(stream: &mut TcpStream, expected: &str) -> R<()> {
    loop {
        let mut buffer = [0u8; 1024];
        match stream.read(&mut buffer).await {
            Err(_) => panic!("Read failed!"),
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                let resp = Parser::new(&buffer).parse().unwrap();
                println!("{}", resp.try_to_string().unwrap());
                assert!(resp.cmp_str(expected))
            }
        }
    }
    Ok(())
}

pub async fn write(stream: &mut TcpStream, msg: String) -> R<()> {
    stream
        .write_all(msg.as_bytes())
        .await
        .expect("Write failed!");
    stream.flush().await.expect("Flush failed!");
    Ok(())
}
