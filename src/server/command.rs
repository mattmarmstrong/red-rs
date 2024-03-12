use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use hashbrown::{HashMap, HashSet};
use lazy_static::lazy_static;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::resp::data::DataType;
use crate::resp::serialize::Serializer;

use super::errors::CommandError;
use super::store::Store;

#[derive(Debug, Eq)]
struct OptionEntry {
    name: String,
    has_arg: bool,
}

impl PartialEq for OptionEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for OptionEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl OptionEntry {
    fn new(name: String, has_arg: bool) -> Self {
        Self { name, has_arg }
    }
}

type OptionsEntry = Option<HashSet<OptionEntry>>;

#[derive(Debug)]
pub struct CommandEntry {
    args: usize, // min. required args
    options: OptionsEntry,
}

impl CommandEntry {
    fn new(args: usize, options: OptionsEntry) -> Self {
        Self { args, options }
    }

    #[inline]
    pub fn has_opts(&self) -> bool {
        self.options.is_some()
    }

    pub fn get_opt_entry(&self, opt: String) -> Option<OptionEntry> {
        if self.options() {
            self.options.unwrap().get(&opt)
        } else {
            None
        }
    }
}

lazy_static! {
    pub static ref COMMANDS: HashMap<String, CommandEntry> = {
        let mut commands = HashMap::new();
        // Command - ping
        let ping_entry = CommandEntry::new(0, None);
        commands.insert("ping".to_string(), ping_entry);
        // Command - echo
        let echo_entry = CommandEntry::new(1, None);
        commands.insert("echo".to_string(), echo_entry);
        // Command - get
        let get_entry = CommandEntry::new(1, None);
        commands.insert("get".to_string(), get_entry);
        // Command - set
        let set_options = HashSet::new();
        let px_entry = OptionEntry::new("px".to_string(), true);
        set_options.insert(px_entry);
        let set_entry = CommandEntry::new(2, Some(set_options));
        commands.insert("set".to_string(), set_entry);

        // Command table
        commands
    };
}

type Vec<T> = VecDeque<T>;
type R<T> = anyhow::Result<T, CommandError>;

struct CommandOption {
    name: String,
    val: Option<String>,
}

impl CommandOption {
    fn new(name: String, val: Option<String>) -> Self {
        Self { name, val }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    PING,
    Echo(String),
    Get(String),
    Set(String, String, Option<Vec<CommandOption>>),
}

impl Command {
    fn parse_options(name: &str, mut args: Vec<String>) -> R<Vec<CommandOption>> {
        // this is utterly disgusting
        if let Some(entry) = COMMANDS.get(name) {
            let mut buffer = Vec::new();
            let n = args.len();
            let mut i = 0;
            while i < n {
                match args.pop_front() {
                    Some(name) => {
                        // accounting for the arg that was just popped.
                        i += 1;
                        match entry.get_opt_entry(name) {
                            Some(opt_entry) => {
                                match !buffer.iter().any(|opt| name == opt.name) {
                                    true => {
                                        match opt_entry.has_arg {
                                            true => {
                                                match args.pop_front() {
                                                    Some(val) => {
                                                        // two args popped
                                                        i += 1;
                                                        let cmd_opt =
                                                            CommandOption::new(name, Some(val));
                                                        // duplicate args
                                                        // valid arg
                                                        buffer.push_back(cmd_opt);
                                                    }
                                                    None => {
                                                        return Err(CommandError::InvalidArgs);
                                                    }
                                                }
                                            }
                                            false => {
                                                let cmd_opt = CommandOption::new(name, None);
                                                buffer.push_back(value);
                                            }
                                        }
                                    }
                                    false => {
                                        return Err(CommandError::InvalidArgs);
                                    }
                                }
                            }
                            None => {
                                return Err(CommandError::InvalidArgs);
                            }
                        }
                    }
                    None => {
                        return Err(CommandError::InvalidArgs);
                    }
                }
            }
            Ok(buffer)
        } else {
            Err(CommandError::NotFound)
        }
    }

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
        match (key, val) {
            (Some(k), Some(v)) => {
                if args.len() > 0 {
                    let options = Command::parse_options("set", args)?;
                    Ok(Command::Set(k, v, Some(options)))
                } else {
                    Ok(Command::Set(k, v, None))
                }
            }
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

    fn do_set(
        key: &String,
        val: &String,
        options: Option<Vec<CommandOption>>,
        store: &mut Store,
    ) -> String {
        let exp: Option<Duration>;
        match options {
            Some(opts) => {
                // TODO: Support other options
                match opts.iter().find(|opt| &opt.name == "px") {
                    Some(opt) => {
                        // this is handled before this point, unwrapping is safe
                        match opt.val.unwrap().parse::<usize>() {
                            Ok(dur) => {
                                exp = Some(Duration::from_millis(dur));
                            }
                            Err(_) => panic!(),
                        }
                    }
                    None => exp = None,
                }
            }
            None => exp = None,
        }
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
        match COMMANDS.contains_key(&first) {
            true => {
                if str_arr.len() > 0 {
                    return Ok(Self::try_new(&first, Some(str_arr))?);
                } else {
                    return Ok(Self::try_new(&first, None)?);
                }
            }
            false => Err(CommandError::NotFound),
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
