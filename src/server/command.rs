use std::collections::VecDeque;
use std::time::Duration;

use hashbrown::HashMap;
use lazy_static::lazy_static;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::resp::data::DataType;
use crate::resp::serialize::Serializer;

use super::errors::CommandError;
use super::store::Store;

// Eventually, this should have a function for every individual command.
// Then we can dispatch actions accordingly

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    PING,
    Echo(String),
    Get(String),
    Set(String, String, Option<Duration>),
}

impl Command {
    fn ping() -> Result<Self, CommandError> {
        Ok(Self::PING)
    }

    fn echo(mut args: Vec<String>) -> R<Self> {
        // errors should be handled well before this point
        // leaving this here for now, will remove later
        match args.pop_front() {
            Some(arg) => Ok(Self::Echo(arg)),
            None => Err(CommandError::InvalidArgs),
        }
    }

    fn get(mut args: Vec<String>) -> R<Self> {
        match args.pop_front() {
            Some(arg) => Ok(Self::Get(arg)),
            None => Err(CommandError::InvalidArgs),
        }
    }

    fn set(mut args: Vec<String>) -> R<Self> {
        let key = args.pop_front();
        let val = args.pop_front();
        let exp = args.pop_front();
        match (key, val) {
            (Some(k), Some(v)) => match exp {
                Some(dur) => match dur.parse::<u64>() {
                    Ok(ms) => {
                        let expiry = Some(Duration::from_millis(ms));
                        Ok(Self::Set(k, v, expiry))
                    }
                    Err(_) => Err(CommandError::InvalidArgs),
                },
                None => Ok(Self::Set(k, v, None)),
            },
            _ => Err(CommandError::InvalidArgs),
        }
    }

    #[inline]
    fn do_ping() -> String {
        "+PONG\r\n".to_string()
    }

    #[inline]
    fn do_echo(arg: &String) -> String {
        arg.to_owned()
    }

    fn do_get(key: &String, store: &Store) -> String {
        match store.try_read(key.to_owned()) {
            Ok(val) => match val {
                Some(v) => v,
                None => "$-1\r\n".to_string(),
            },
            // TODO: handle read errors more gracefully
            Err(_) => "$-1\r\n".to_string(),
        }
    }

    fn do_set(key: &String, val: &String, exp: &Option<Duration>, store: &mut Store) -> String {
        match store.try_write(key.to_owned(), val.to_owned(), exp.to_owned()) {
            Ok(_) => "+OK\r\n".to_string(),
            // TODO: error handling
            Err(_) => "$-1\r\n".to_string(),
        }
    }

    fn try_new(str: &str, args: Option<Vec<String>>) -> R<Self> {
        match str {
            // No args commands
            "ping" => Command::ping(),
            _ => {
                // args commands
                let args = args.unwrap();
                match str {
                    "echo" => Command::echo(args),
                    "get" => Command::get(args),
                    "set" => Command::set(args),
                    _ => Err(CommandError::NotFound),
                }
            }
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
            .map(|data| {
                let s = data.try_to_string().unwrap();
                println!("Array arg: {}", s);
                s
            })
            .collect::<Vec<String>>();
        let first = str_arr.pop_front().unwrap();
        // COMMANDS currently stores only the number of required args.
        // This should change.
        match COMMANDS.get(&first) {
            Some(_) => {
                if str_arr.len() > 0 {
                    return Ok(Self::try_new(&first, Some(str_arr))?);
                } else {
                    return Ok(Self::try_new(&first, None)?);
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
                resp = Command::do_ping();
            }
            Self::Echo(s) => {
                resp = Serializer::to_simple_str(&Command::do_echo(s));
            }
            Self::Get(key) => {
                resp = Serializer::to_bulk_str(&Command::do_get(key, store));
            }
            Self::Set(k, v, exp) => {
                resp = Command::do_set(k, v, exp, store);
            }
        }
        stream.write_all(resp.as_bytes()).await?;
        Ok(())
    }
}
