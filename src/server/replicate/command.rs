use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::resp::serialize::Serializer;
use crate::server::errors::CommandError;
use crate::server::Server;

use super::info::Role;

// Put code to communicate between replicas here
type R<T> = anyhow::Result<T, CommandError>;

pub async fn do_repl_handshake(server: &Server) -> R<()> {
    // TODO -> The rest of the repl_handshake
    debug_assert!(server.replica_info.role == Role::Slave);
    debug_assert!(server.master_addr().is_some());
    match TcpStream::connect(server.master_addr().unwrap()).await {
        Ok(mut master_stream) => {
            let ping_str = Serializer::to_bulk_str("ping");
            let ping = Serializer::to_arr(Vec::from([ping_str]));
            master_stream
                .write_all(ping.as_bytes())
                .await
                .expect("Failed to write!");
            Ok(())
        }
        Err(_) => Err(CommandError::CommandFailed),
    }
}
