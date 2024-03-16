#[derive(Debug)]
pub enum ReplError {
    FailedToConnect,
    InvalidResponse,
    UnexpectedResponse,
    MasterHandshakeFailed,
}

impl std::fmt::Display for ReplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedToConnect => {
                write!(f, "REPL Error: Failed to connect!")
            }
            Self::InvalidResponse => {
                write!(f, "REPL Error: Response not in valid RESP format!")
            }
            Self::UnexpectedResponse => {
                write!(f, "REPL Error: Unexpected response!")
            }
            Self::MasterHandshakeFailed => {
                write!(f, "REPL Error: Master handshake failed!")
            }
        }
    }
}

impl std::error::Error for ReplError {}
