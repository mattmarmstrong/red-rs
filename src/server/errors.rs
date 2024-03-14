#[derive(Debug)]
pub enum CommandError {
    NotFound,
    InvalidArgs,
    InvalidOption,
    CommandFailed,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => {
                write!(f, "Command Error: Not found!")
            }
            Self::InvalidArgs => {
                write!(f, "Command Error: Invalid args!")
            }
            Self::InvalidOption => {
                write!(f, "Command Error: Invalid option!")
            }
            Self::CommandFailed => {
                write!(f, "Command Error: Command failed!")
            }
        }
    }
}

impl std::error::Error for CommandError {}

#[derive(Debug)]
pub enum StoreError {
    ReadFailed,
    WriteFailed,
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
        }
    }
}

impl std::error::Error for StoreError {}
