pub mod command;
pub mod errors;
pub mod store;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::resp::parse::Parser;

use command::Command;
use store::Store;

pub async fn handle_connection(mut stream: &mut TcpStream, mut store: Store) -> anyhow::Result<()> {
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .expect("Failed to read from client stream!");
        if bytes_read == 0 {
            break;
        }

        let mut parser = Parser::new(&buffer);
        let data = parser.parse()?;
        if let Some(cmd) = Command::new(data) {
            cmd.execute(&mut stream, &mut store).await?;
        }
    }
    Ok(())
}
