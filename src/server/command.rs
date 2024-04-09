// This file is intended to include leader -> follower commands
// The follower -> leader commands should be in server/replicate
use std::borrow::Borrow;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use hashbrown::{HashMap, HashSet};
use lazy_static::lazy_static;
use regex::Regex;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

use crate::resp::data::DataType;
use crate::resp::serialize::Serializer;

use super::errors::CommandError;
use super::store::errors::StoreError;
use super::store::file::empty_store_file_bytes;
use super::Server;

#[derive(Debug, Eq)]
struct OptionEntry {
    name: String,
    args: Option<usize>,
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

impl Borrow<String> for OptionEntry {
    fn borrow(&self) -> &String {
        &self.name
    }
}

impl OptionEntry {
    fn new(name: String, args: Option<usize>) -> Self {
        Self { name, args }
    }
}

type OptionEntries = Option<HashSet<OptionEntry>>;

#[derive(Debug)]
pub struct CommandEntry {
    args: usize, // min. required args
    options: OptionEntries,
}

impl CommandEntry {
    fn new(args: usize, options: OptionEntries) -> Self {
        Self { args, options }
    }

    #[inline]
    fn options(&self) -> bool {
        self.options.is_some()
    }

    fn get_option_entry(&self, opt: &String) -> Option<&OptionEntry> {
        if self.options() {
            self.options.as_ref().unwrap().get(opt)
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
        let mut set_options = HashSet::new();
        let px_entry = OptionEntry::new("px".to_string(), Some(1));
        set_options.insert(px_entry);
        let set_entry = CommandEntry::new(2, Some(set_options));
        commands.insert("set".to_string(), set_entry);

        // Command - info
        let mut info_options = HashSet::new();
        let repl_entry = OptionEntry::new("replication".to_string(), None);
        info_options.insert(repl_entry);
        let info_entry = CommandEntry::new(1, Some(info_options));
        commands.insert("info".to_string(), info_entry);

        // Command - replconf
        let mut repl_conf_options = HashSet::new();
        let listen_entry = OptionEntry::new("listening-port".to_string(), Some(1));
        let capa_entry = OptionEntry::new("capa".to_string(), Some(1));
        repl_conf_options.insert(listen_entry);
        repl_conf_options.insert(capa_entry);
        let repl_conf_entry = CommandEntry::new(1, Some(repl_conf_options));
        commands.insert("replconf".to_string(), repl_conf_entry);

        // Command - psync
        let psync_options = HashSet::new();
        let psync_entry = CommandEntry::new(2, Some(psync_options));
        commands.insert("psync".to_string(), psync_entry);

        // Command - type
        let type_entry = CommandEntry::new(1, None);
        commands.insert("type".to_string(), type_entry);

        // Command - xadd
        let xadd_entry = CommandEntry::new(2, None);
        commands.insert("xadd".to_string(), xadd_entry);

        commands
    };
}

type Vec<T> = VecDeque<T>;
type R<T> = anyhow::Result<T, CommandError>;

#[derive(Debug)]
pub struct CommandOption {
    name: String,
    val: Option<Vec<String>>,
}

impl CommandOption {
    fn new(name: String, val: Option<Vec<String>>) -> Self {
        Self { name, val }
    }
}

#[derive(Debug, PartialEq)]
pub enum CommandResult {
    Ok,
    Err,
    ReplConf,
}

#[derive(Debug)]
pub enum Command {
    PING,
    PSync(String, isize),
    Echo(String),
    Get(String),
    Set {
        key: String,
        val: String,
        px: Option<Duration>,
    },
    Info(String),
    ReplConf {
        port: Option<u16>,
        capa: Option<String>,
    },
    Tipe(String),
    XAdd {
        key: String,
        id: Option<(usize, usize)>,
        values: Vec<(String, String)>,
    },
}

impl Command {
    fn parse_options(name: &str, mut args: Vec<String>) -> R<Vec<CommandOption>> {
        let mut buffer = Vec::new();
        // it's on the caller to ensure that this won't panic
        let entry = COMMANDS.get(name).unwrap();
        while let Some(arg) = args.pop_front() {
            match entry.get_option_entry(&arg) {
                Some(o_entry) => {
                    if let Some(n) = o_entry.args {
                        if n > args.len() {
                            return Err(CommandError::InvalidArgs);
                        };
                        let mut vals = Vec::with_capacity(n);
                        for _ in 0..n {
                            vals.push_back(args.pop_front().unwrap())
                        }
                        let cmd_o = CommandOption::new(o_entry.name.to_owned(), Some(vals));
                        buffer.push_back(cmd_o);
                    } else {
                        let cmd_o = CommandOption::new(o_entry.name.to_owned(), None);
                        buffer.push_back(cmd_o);
                    }
                }
                None => return Err(CommandError::InvalidOption),
            }
        }
        Ok(buffer)
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
        let k = args.pop_front();
        let v = args.pop_front();
        match (k, v) {
            (Some(key), Some(val)) => {
                if !args.is_empty() {
                    let mut options = Command::parse_options("set", args)?;
                    // one arg for this right now, this is fine. Should do a for_each and map each
                    // option to it's named counterpart.
                    let px_o = options.pop_front().unwrap();
                    let px_val = px_o.val.unwrap().pop_front().unwrap();
                    match px_val.parse::<u64>() {
                        Ok(px) => Ok(Command::Set {
                            key,
                            val,
                            px: Some(Duration::from_millis(px)),
                        }),
                        Err(_) => Err(CommandError::InvalidArgs),
                    }
                } else {
                    Ok(Command::Set { key, val, px: None })
                }
            }
            _ => Err(CommandError::InvalidArgs),
        }
    }

    fn tipe(mut args: Vec<String>) -> R<Self> {
        print!("Called type cmd constructor");
        match args.pop_front() {
            Some(k) => Ok(Self::Tipe(k)),
            None => Err(CommandError::InvalidArgs),
        }
    }

    fn info(args: Vec<String>) -> R<Self> {
        // TODO: refactor
        let mut options = Command::parse_options("info", args)?;
        debug_assert!(options.len() == 1);
        let option = options.pop_front().unwrap();
        Ok(Self::Info(option.name))
    }

    fn repl_conf(args: Vec<String>) -> R<Self> {
        let mut options = Command::parse_options("replconf", args)?;
        let mut port: Option<u16> = None;
        let mut capa: Option<String> = None;
        debug_assert!(options.len() == 1);
        let opt = options.pop_front().unwrap();
        match opt.name.as_str() {
            "listening-port" => {
                port = opt.val.unwrap().pop_front().unwrap().parse::<u16>().ok();
            }
            "capa" => {
                capa = opt.val.unwrap().pop_front();
            }
            _ => return Err(CommandError::InvalidOption),
        }
        Ok(Self::ReplConf { port, capa })
    }

    fn psync(mut args: Vec<String>) -> R<Self> {
        let repl_id = args.pop_front();
        let offset = args.pop_front();
        match (repl_id, offset) {
            (Some(id), Some(offset)) => {
                if let Ok(o) = offset.parse::<isize>() {
                    Ok(Self::PSync(id, o))
                } else {
                    Err(CommandError::InvalidArgs)
                }
            }
            _ => Err(CommandError::InvalidArgs),
        }
    }

    fn xadd(mut args: Vec<String>) -> R<Self> {
        let key = args.pop_front().unwrap();
        // stream key regex
        let re = Regex::new(r"[0-9]+-[0-9]+").unwrap();
        let stream_id: Option<(usize, usize)>;
        let k: String;
        let next = args.pop_front().unwrap();
        if re.is_match(&next) {
            // Fix me
            let mut split = next.split('-');
            let id = split
                .next()
                .unwrap()
                .parse::<usize>()
                .expect("Could not parse ID!");
            let seq = split
                .next()
                .unwrap()
                .parse::<usize>()
                .expect("Could not parse sequence!");
            stream_id = Some((id, seq));
            k = args.pop_front().unwrap();
        } else {
            stream_id = None;
            k = next
        }
        let mut buffer = Vec::with_capacity(8);
        let v = args.pop_front().unwrap();
        buffer.push_back((k, v));
        let remaining_args = args.len();
        // The way this is written is extremely confusing, but since the first stream key has technically been popped
        // already, this should be an odd number.
        match remaining_args % 2 == 0 {
            true => {
                for _ in (0..remaining_args).step_by(2) {
                    let k = args.pop_front().unwrap();
                    let v = args.pop_front().unwrap();
                    buffer.push_back((k, v))
                }
                Ok(Self::XAdd {
                    key,
                    id: stream_id,
                    values: buffer,
                })
            }
            false => Err(CommandError::InvalidArgs),
        }
    }

    #[inline]
    async fn do_ping(stream: &mut TcpStream) -> R<CommandResult> {
        stream
            .write_all(b"+PONG\r\n")
            .await
            .expect("Response write failed!");
        Ok(CommandResult::Ok)
    }

    #[inline]
    async fn do_echo(arg: &str, stream: &mut TcpStream) -> R<CommandResult> {
        stream
            .write_all(Serializer::to_simple_str(arg).as_bytes())
            .await
            .expect("Response write failed!");
        Ok(CommandResult::Ok)
    }

    async fn do_get(
        key: String,
        server: &Arc<RwLock<Server>>,
        stream: &mut TcpStream,
    ) -> R<CommandResult> {
        let s = server.read().await;
        let resp = match s.store.kv_store.try_read(key.to_owned()) {
            Some(v) => Serializer::to_bulk_str(&v),
            None => "$-1\r\n".to_string(),
        };
        stream
            .write_all(resp.as_bytes())
            .await
            .expect("Response write failed!");
        Ok(CommandResult::Ok)
    }

    async fn do_set(
        key: String,
        val: String,
        exp: Option<Duration>,
        server: &Arc<RwLock<Server>>,
        stream: &mut TcpStream,
    ) -> R<CommandResult> {
        let mut s = server.write().await;
        let resp = match s.store.kv_store.try_write(key.clone(), val.clone(), exp) {
            Ok(_) => "+OK\r\n",
            // TODO: error handling
            Err(_) => "$-1\r\n",
        };
        stream
            .write_all(resp.as_bytes())
            .await
            .expect("Response write failed!");
        if s.replicas.is_some() {
            // re-constructing the byte slice we recieved then deconstructed. big brain move
            // refactor me!
            let mut cmd_vec = std::vec::Vec::with_capacity(5);
            cmd_vec.push("set");
            cmd_vec.push(&key);
            cmd_vec.push(&val);
            // skipping the px command for now, this will bite later I'm sure.
            let cmd = Serializer::to_arr(cmd_vec);
            if s.repl_queue.is_some() {
                s.repl_queue.as_mut().unwrap().push(cmd);
            } else {
                s.repl_queue = Some(vec![cmd]);
            }
        }
        Ok(CommandResult::Ok)
    }

    async fn do_tipe(
        key: String,
        server: &Arc<RwLock<Server>>,
        stream: &mut TcpStream,
    ) -> R<CommandResult> {
        let read = server.read().await;
        let resp = match read.store.kv_store.try_read(key.to_owned()) {
            Some(_) => Serializer::to_simple_str("string"),
            None => match read.store.stream_store.try_read(key) {
                Some(_) => Serializer::to_simple_str("stream"),
                None => Serializer::to_simple_str("none"),
            },
        };
        stream
            .write_all(resp.as_bytes())
            .await
            .expect("Response write failed!");
        Ok(CommandResult::Ok)
    }

    async fn do_info(
        info_type: &str,
        server: &Arc<RwLock<Server>>,
        stream: &mut TcpStream,
    ) -> R<CommandResult> {
        let s = server.read().await;
        let resp = match info_type {
            "replication" => Serializer::to_bulk_str(&s.replica_info.to_string()),
            _ => todo!(),
        };
        stream
            .write_all(resp.as_bytes())
            .await
            .expect("Response write failed!");
        Ok(CommandResult::Ok)
    }

    async fn do_repl_conf(_port: Option<u16>, stream: &mut TcpStream) -> R<CommandResult> {
        let resp = Serializer::to_simple_str("OK");
        stream
            .write_all(resp.as_bytes())
            .await
            .expect("Response write failed!");
        Ok(CommandResult::Ok)
    }

    async fn do_psync(
        repl_id: String,
        server: &Arc<RwLock<Server>>,
        stream: &mut TcpStream,
    ) -> R<CommandResult> {
        let s = server.read().await;
        let master_replid = s.replica_info.master_replid.as_ref().unwrap();
        let master_repl_offset = s.replica_info.master_repl_offset.to_string();
        let repl_command = match repl_id.as_str() {
            "?" => "FULLRESYNC",
            _ => unimplemented!(),
        };
        let command_str = [repl_command, " ", &master_replid, " ", &master_repl_offset].concat();
        let resync = Serializer::to_simple_str(&command_str);
        stream
            .write_all(resync.as_bytes())
            .await
            .expect("Failed to write!");
        let store_file = Serializer::serialize_store_file(empty_store_file_bytes());
        stream
            .write_all(store_file.as_slice())
            .await
            .expect("Failed to write!");
        Ok(CommandResult::ReplConf)
    }

    async fn do_xadd(
        key: String,
        values: Vec<(String, String)>,
        stream_id: Option<(usize, usize)>,
        server: &Arc<RwLock<Server>>,
        stream: &mut TcpStream,
    ) -> R<CommandResult> {
        let mut s = server.write().await;
        let resp = match s.store.stream_store.try_write(key, values, stream_id) {
            Ok((id, seq)) => Serializer::to_bulk_str(
                &[id.to_string().as_str(), "-", seq.to_string().as_str()].concat(),
            ),
            Err(e) => match e {
                StoreError::StreamIDZero => {
                    Serializer::to_simple_err("The ID specified in XADD must be greater than 0-0")
                }
                StoreError::InvalidStreamID => Serializer::to_simple_err(
                    "The ID specified in XADD is equal or smaller than the target stream top item",
                ),
                _ => Serializer::to_simple_err("Unknown error occurred"),
            },
        };
        stream
            .write_all(resp.as_bytes())
            .await
            .expect("Response write failed!");
        Ok(CommandResult::Ok)
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
                    "info" => Command::info(args),
                    "replconf" => Command::repl_conf(args),
                    "psync" => Command::psync(args),
                    "type" => Command::tipe(args),
                    "xadd" => Command::xadd(args),
                    _ => Err(CommandError::NotFound),
                }
            }
        }
    }

    fn from_str(s: String) -> R<Self> {
        match COMMANDS.get(&s) {
            Some(entry) => match entry.args {
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
        match COMMANDS.contains_key(&first) {
            true => {
                if !str_arr.is_empty() {
                    Self::try_new(&first, Some(str_arr))
                } else {
                    Self::try_new(&first, None)
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

    pub async fn execute(
        self,
        stream: &mut TcpStream,
        server: &Arc<RwLock<Server>>,
    ) -> R<CommandResult> {
        match self {
            Self::PING => Command::do_ping(stream).await,
            Self::Echo(s) => Command::do_echo(s.as_str(), stream).await,
            Self::Get(key) => Command::do_get(key, server, stream).await,
            Self::Set { key, val, px } => Command::do_set(key, val, px, server, stream).await,
            Self::Info(v) => Command::do_info(v.as_str(), server, stream).await,
            Self::ReplConf { port, capa: _ } => Command::do_repl_conf(port, stream).await,
            Self::PSync(repl_id, _) => Command::do_psync(repl_id, server, stream).await,
            Self::Tipe(key) => Command::do_tipe(key, server, stream).await,
            Self::XAdd { key, id, values } => {
                Command::do_xadd(key, values, id, server, stream).await
            }
        }
    }
}
