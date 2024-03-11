use lazy_static::lazy_static;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use std::collections::{HashMap, VecDeque};

use crate::resp::data::DataType;
use crate::resp::serialize::Serializer;

use super::errors::CommandError;
use super::Store;

lazy_static! {
    // key: command name
    // value: number of args
    pub static ref COMMANDS: HashMap<String, usize> = {
        let mut commands = HashMap::new();
        commands.insert("ping".to_string(), 0);
        commands.insert("echo".to_string(), 1);
        commands.insert("get".to_string(), 1);
        commands.insert("set".to_string(), 2);
        commands
    };
}

type Vec<T> = VecDeque<T>;
type R<T> = anyhow::Result<T, CommandError>;

#[derive(Debug, Clone)]
pub enum Command {
    PING,
    Echo(String),
    Get(String),
    Set(String, String),
}

impl Command {
    // This function assumes that the args slice passed in is of the correct length
    fn try_new(str: &str, args: Option<Vec<String>>) -> R<Self> {
        match str {
            "ping" => Ok(Self::PING),
            "echo" => {
                let arg = args.unwrap().pop_front().unwrap();
                Ok(Self::Echo(arg))
            }
            "get" => {
                let key = args.unwrap().pop_front().unwrap();
                Ok(Self::Get(key))
            }
            "set" => {
                let mut args = args.unwrap();
                let key = args.pop_front().unwrap();
                let val = args.pop_front().unwrap();
                Ok(Self::Set(key.to_owned(), val.to_owned()))
            }
            _ => Err(CommandError::NotFound),
        }
    }

    fn from_str(s: String) -> R<Self> {
        match COMMANDS.get(&s) {
            Some(&n) => match &n {
                0 => Ok(Self::try_new(&s, None)?),
                _ => Err(CommandError::InvalidArgs),
            },
            None => Err(CommandError::NotFound),
        }
    }

    fn from_arr(arr: Vec<DataType>) -> R<Self> {
        // Currently assumes that the array args are all string types.
        let mut str_arr = arr
            .iter()
            .map(|data| data.try_to_string().unwrap())
            .collect::<Vec<String>>();
        let first = str_arr.pop_front().unwrap();
        // COMMANDS currently stores only the number of required args.
        // This should change.
        match COMMANDS.get(&first) {
            Some(&n) => {
                let actual = arr.len() - 1;
                match actual == n {
                    true => {
                        if n > 0 {
                            return Ok(Self::try_new(&first, Some(str_arr))?);
                        } else {
                            return Ok(Self::try_new(&first, None)?);
                        }
                    }
                    false => Err(CommandError::InvalidArgs),
                }
            }
            None => Err(CommandError::NotFound),
        }
    }

    pub fn new(data: DataType) -> Option<Self> {
        match data {
            DataType::SimpleString(s) | DataType::BulkString(s) => Self::from_str(s).ok(),
            DataType::Array(arr) => Self::from_arr(arr).ok(),
            _ => unreachable!(), // This shouldn't get called on other types
        }
    }

    pub async fn execute(&self, stream: &mut TcpStream, store: &mut Store) -> anyhow::Result<()> {
        let resp: String;
        match self {
            Self::PING => {
                resp = "+PONG\r\n".to_string();
            }
            Self::Echo(s) => {
                resp = Serializer::to_simple_str(&s);
            }
            Self::Get(key) => {
                resp = Serializer::to_bulk_str(&key);
            }
            Self::Set(key, val) => {
                store
                    .write()
                    .unwrap()
                    .insert(key.to_owned(), val.to_owned());
                resp = "+OK\r\n".to_string();
            }
        }
        stream.write_all(resp.as_bytes()).await?;
        Ok(())
    }
}
