use std::collections::HashSet;

use lazy_static::lazy_static;

use super::data::DataType;

lazy_static! {
    // TODO: return to this
    pub static ref COMMANDS: HashSet<String> = {
        let mut commands = HashSet::new();
        commands.insert("echo".to_string());
        commands.insert("ping".to_string());
        commands
    };
}

pub enum Command {
    Echo(String),
    PING,
}

impl Command {
    pub fn from_dt(dt: &DataType) -> Option<Self> {
        match dt {
            DataType::SimpleString(s) => match COMMANDS.contains(s) {
                true => match s.as_str() {
                    "ping" => Some(Command::PING),
                    _ => unimplemented!(),
                },
                false => None,
            },
            DataType::BulkString(s) => match COMMANDS.contains(s) {
                true => match s.as_str() {
                    "ping" => Some(Command::PING),
                    _ => unimplemented!(),
                },
                false => None,
            },
            DataType::Array(arr) => {
                let first = &arr[0];
                if first.cmp_str("echo") {
                    return Some(Command::Echo(arr[1].try_to_string().unwrap()));
                };
                if first.cmp_str("ping") {
                    return Some(Command::PING);
                };
                None
            }
            _ => unimplemented!(),
        }
    }

    pub fn response(&self) -> Option<&[u8]> {
        match self {
            Command::PING => Some(b"+PONG\r\n"),
            Command::Echo(s) => Some(s.as_bytes()),
        }
    }
}
