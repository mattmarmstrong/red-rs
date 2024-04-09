#[derive(Debug)]
pub enum StoreError {
    ReadFailed,
    WriteFailed,
    InvalidStreamID,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFailed => {
                write!(f, "Store Error: Read failed!")
            }
            Self::WriteFailed => {
                write!(f, "Store Error: Write failed!")
            }
            Self::InvalidStreamID => {
                write!(f, "Store Error: Invalid stream ID!")
            }
        }
    }
}

impl std::error::Error for StoreError {}
