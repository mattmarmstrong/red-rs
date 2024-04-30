#[derive(Debug)]
pub enum StreamError {
    StreamIDZero,
    InvalidStreamID,
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Should move these
            Self::StreamIDZero => {
                write!(f, "Store Error: Stream ID must not be zero!")
            }
            Self::InvalidStreamID => {
                write!(f, "Store Error: Invalid stream ID!")
            }
        }
    }
}

impl std::error::Error for StreamError {}
