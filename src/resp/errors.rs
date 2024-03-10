#[derive(Debug)]
pub enum RESPError {
    InvalidType,
    InvalidData,
}

impl std::fmt::Display for RESPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidType => {
                write!(f, "RESP Error: Invalid type!")
            }
            Self::InvalidData => {
                write!(f, "RESP Error: Invalid data!")
            }
        }
    }
}

impl std::error::Error for RESPError {}
