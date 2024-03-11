use std::collections::VecDeque;

use super::data::DataType;
use super::errors::RESPError;

// RESP sequence terminal characters. -> https://redis.io/docs/reference/protocol-spec/
pub const _CRLF: &[u8] = b"\r\n";

type Vec<T> = VecDeque<T>;
type R<T> = anyhow::Result<T, RESPError>;

#[derive(Debug)]
pub struct Parser<'data> {
    data: &'data [u8],
    position: usize,
    len: usize,
}

impl<'data> Parser<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        Self {
            data,
            position: 0,
            len: data.len(),
        }
    }

    #[inline]
    fn peek(&self) -> u8 {
        self.data[self.position + 1]
    }

    #[inline]
    fn read_byte(&mut self) -> u8 {
        let byte = self.data[self.position];
        self.position += 1;
        byte
    }

    #[inline]
    fn at_end(&self) -> bool {
        self.position == self.len - 1
    }

    // end-of-sequence
    #[inline]
    fn is_eos(&self) -> bool {
        self.data[self.position] == b'\r' && self.peek() == b'\n'
    }

    fn skip_crlf(&mut self) -> anyhow::Result<(), RESPError> {
        match !self.at_end() {
            true => {
                if self.is_eos() {
                    self.read_byte();
                    self.read_byte();
                    return Ok(());
                } else {
                    return Err(RESPError::InvalidData);
                }
            }
            false => Err(RESPError::InvalidData),
        }
    }

    fn parse_len(&mut self) -> anyhow::Result<isize, RESPError> {
        let mut buffer = String::new();
        while !self.is_eos() && !self.at_end() {
            buffer.push(self.read_byte() as char);
        }
        match buffer.parse::<isize>() {
            Ok(len) => Ok(len),
            Err(_) => Err(RESPError::InvalidData),
        }
    }

    fn parse_simple_str(&mut self) -> R<DataType> {
        self.read_byte(); // skip the type byte
        let mut buffer = String::new();
        while !self.is_eos() {
            if !self.at_end() {
                buffer.push(self.read_byte().to_ascii_lowercase() as char)
            } else {
                return Err(RESPError::InvalidData);
            }
        }
        self.skip_crlf()?;
        Ok(DataType::SimpleString(buffer))
    }

    fn parse_bulk_str(&mut self) -> R<DataType> {
        self.read_byte(); // skip the type byte
        let len = self.parse_len()?;
        match len > 0 {
            true => {
                self.skip_crlf()?;
                let mut buffer = String::with_capacity(len as usize);
                for _ in 0..len {
                    if !self.at_end() {
                        buffer.push(self.read_byte().to_ascii_lowercase() as char);
                    } else {
                        return Err(RESPError::InvalidData);
                    }
                }
                self.skip_crlf()?;
                Ok(DataType::BulkString(buffer))
            }
            false => Ok(DataType::BulkString(String::new())),
        }
    }

    fn parse_array(&mut self) -> R<DataType> {
        self.read_byte(); // skip the type byte
        let len = self.parse_len()?;
        match len > 0 {
            true => {
                self.skip_crlf()?;
                let mut buffer = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    buffer.push_back(self.parse()?);
                }
                Ok(DataType::Array(buffer))
            }
            false => Ok(DataType::Array(Vec::new())),
        }
    }

    pub fn parse(&mut self) -> R<DataType> {
        Ok(match self.data[self.position] {
            b'+' => self.parse_simple_str()?,
            b'$' => self.parse_bulk_str()?,
            b'*' => self.parse_array()?,
            _ => return Err(RESPError::InvalidType),
        })
    }
}

#[cfg(test)]
mod tests {

    use std::collections::VecDeque;

    use super::DataType;
    use super::Parser;

    #[test]
    fn test_skip_crlf() {
        let data = b"\r\ntest\r\n";
        let mut parser = Parser::new(data);
        match parser.skip_crlf() {
            Ok(_) => assert_eq!(parser.read_byte(), b't'),
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_parse_len() {
        let data = b"$1234\r\n";
        let mut parser = Parser::new(data);
        let expected = 1234;
        // skiping the type byte, which is what the type parsing functions do already.
        parser.read_byte();
        let actual = parser.parse_len().unwrap();
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_parse_simple_str() {
        let data = b"+TEST\r\n";
        let mut parser = Parser::new(data);
        let expected = DataType::SimpleString("test".to_string());
        let actual = parser.parse().unwrap();
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_parse_bulk_str() {
        let data = b"$4\r\nTEST\r\n";
        let mut parser = Parser::new(data);
        let expected = DataType::BulkString("test".to_string());
        let actual = parser.parse().unwrap();
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_parse_array() {
        let data = b"*2\r\n$4\r\ntest\r\n+TEST\r\n";
        let mut parser = Parser::new(data);
        let mut expected_vec = VecDeque::new();
        expected_vec.push_back(DataType::BulkString("test".to_string()));
        expected_vec.push_back(DataType::SimpleString("test".to_string()));
        let expected = DataType::Array(expected_vec);
        let actual = parser.parse().unwrap();
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_parse_nested_array() {
        let data = b"*2\r\n*2\r\n+OK\r\n+TEST\r\n$4\r\nTEST\r\n";
        let mut parser = Parser::new(data);
        let mut nested_vec = VecDeque::new();
        nested_vec.push_back(DataType::SimpleString("ok".to_string()));
        nested_vec.push_back(DataType::SimpleString("test".to_string()));
        let nested_arr = DataType::Array(nested_vec);
        let mut outer_vec = VecDeque::new();
        outer_vec.push_back(nested_arr);
        outer_vec.push_back(DataType::BulkString("test".to_string()));
        let expected = DataType::Array(outer_vec);
        let actual = parser.parse_array().unwrap();
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_parse_empty() {
        let data = b"$0\r\n\r\n";
        let mut parser = Parser::new(data);
        let expected = DataType::BulkString("".to_string());
        let actual = parser.parse().unwrap();
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_invalid_parse_returns_err() {
        let data = b"INVALID";
        let mut parser = Parser::new(data);
        assert!(parser.parse().is_err());
    }

    #[test]
    fn test_parse_no_crlf_returns_err() {
        let data = b"*2\r\n+TEST";
        let mut parser = Parser::new(data);
        assert!(parser.parse().is_err());
    }

    #[test]
    fn test_invalid_len_returns_err() {
        let data = b"$1A5\rtest\n";
        let mut parser = Parser::new(data);
        assert!(parser.parse_len().is_err())
    }
}
