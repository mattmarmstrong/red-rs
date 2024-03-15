use std::collections::VecDeque;

use super::errors::RESPError;

type Vec<T> = VecDeque<T>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    SimpleString(String),
    SimpleError(String),
    BulkString(String),
    BulkError(String),
    Array(Vec<DataType>),
}

impl DataType {
    pub fn is_str(self, resp: &str) -> bool {
        match self {
            DataType::SimpleString(s) | DataType::BulkString(s) => s.as_str() == resp,
            DataType::Array(mut arr) => {
                if arr.len() == 1 {
                    DataType::is_str(arr.pop_front().unwrap(), resp)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

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

    pub fn try_to_string(&self) -> anyhow::Result<String, RESPError> {
        match self {
            DataType::SimpleString(s) => Ok(s.to_string()),
            DataType::BulkString(s) => Ok(s.to_string()),
            _ => Err(RESPError::InvalidType),
        }
    }
}
