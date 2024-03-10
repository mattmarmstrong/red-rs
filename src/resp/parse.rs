use super::data::DataType;
use super::errors::RESPError;

// RESP sequence terminal characters. -> https://redis.io/docs/reference/protocol-spec/
pub const _CRLF: &[u8] = b"\r\n";

type R<T> = Result<T, RESPError>;

#[derive(Debug)]
pub struct Parser<'data> {
    data: &'data [u8],
    position: usize,
}

impl<'data> Parser<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        Self { data, position: 0 }
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.data[self.position];
        self.position += 1;
        byte
    }

    fn peek(&self) -> u8 {
        self.data[self.position + 1]
    }

    // end-of-sequence
    fn is_eos(&self) -> bool {
        self.data[self.position] == b'\r' && self.peek() == b'\n'
    }

    fn skip_crlf(&mut self) -> Result<(), RESPError> {
        if self.is_eos() {
            self.read_byte();
            self.read_byte();
            return Ok(());
        } else {
            return Err(RESPError::InvalidData);
        }
    }

    fn parse_len(&mut self) -> Result<isize, RESPError> {
        let mut buffer = String::new();
        while !self.is_eos() {
            buffer.push(self.read_byte() as char);
        }
        match buffer.parse::<isize>() {
            Ok(len) => Ok(len),
            Err(_) => Err(RESPError::InvalidData),
        }
    }

    fn parse_simple_str(&mut self) -> R<DataType> {
        let mut buffer = String::new();
        while !self.is_eos() {
            buffer.push(self.read_byte().to_ascii_lowercase() as char)
        }
        self.skip_crlf()?;
        Ok(DataType::SimpleString(buffer))
    }

    fn parse_bulk_str(&mut self) -> R<DataType> {
        let len = self.parse_len()?;
        match len > 0 {
            true => {
                self.skip_crlf()?;
                let mut buffer = String::with_capacity(len as usize);
                for _ in 0..len {
                    buffer.push(self.read_byte().to_ascii_lowercase() as char);
                }
                self.skip_crlf()?;
                Ok(DataType::BulkString(buffer))
            }
            false => Ok(DataType::BulkString(String::new())),
        }
    }

    fn parse_array(&mut self) -> R<DataType> {
        let len = self.parse_len()?;
        match len > 0 {
            true => {
                self.skip_crlf()?;
                let mut buffer = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    buffer.push(self.parse()?);
                }
                self.skip_crlf()?;
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
