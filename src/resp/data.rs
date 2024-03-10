use super::errors::RESPError;

#[derive(Debug)]
pub enum DataType {
    SimpleString(String),
    SimpleError(String),
    BulkString(String),
    BulkError(String),
    Array(Vec<DataType>),
}

impl DataType {
    pub fn cmp_str(&self, cmp: &str) -> bool {
        let a = match self {
            DataType::SimpleString(s) => s,
            DataType::SimpleError(s) => s,
            DataType::BulkString(s) => s,
            DataType::BulkError(s) => s,
            _ => return false,
        };

        a == cmp
    }

    pub fn try_to_string(&self) -> Result<String, RESPError> {
        match self {
            DataType::SimpleString(s) => Ok(s.to_string()),
            DataType::SimpleError(s) => Ok(s.to_string()),
            DataType::BulkString(s) => Ok(s.to_string()),
            DataType::BulkError(s) => Ok(s.to_string()),
            _ => Err(RESPError::InvalidType),
        }
    }
}
