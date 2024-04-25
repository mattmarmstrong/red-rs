use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use hashbrown::HashMap;

use errors::StoreError;

pub mod errors;
pub mod file;

type R<T> = anyhow::Result<T, StoreError>;
type Vec<T> = VecDeque<T>;

#[derive(Debug)]
pub struct KVStoreValue {
    val: String,
    expiry: Option<Instant>,
}

impl KVStoreValue {
    fn new(val: String, exp: Option<Duration>) -> Self {
        let expiry: Option<Instant> = exp.map(|exp| Instant::now() + exp);
        Self { val, expiry }
    }

    #[inline]
    fn is_expired(&self) -> bool {
        match self.expiry {
            Some(exp) => exp < Instant::now(),
            None => false,
        }
    }

    fn to_val(&self) -> Option<String> {
        match self.is_expired() {
            false => Some(self.val.to_owned()),
            true => None,
        }
    }
}

#[derive(Debug)]
pub struct KVStore {
    pub inner: HashMap<String, KVStoreValue>,
}

impl Default for KVStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KVStore {
    fn new() -> Self {
        let inner = HashMap::new();
        Self { inner }
    }

    pub fn try_read(&self, key: String) -> Option<String> {
        match self.inner.get(&key) {
            Some(store_val) => store_val.to_val(),
            None => None,
        }
    }

    pub fn try_write(&mut self, key: String, val: String, exp: Option<Duration>) -> R<()> {
        let store_val = KVStoreValue::new(val, exp);
        if self.inner.get(&key).is_some() {
            self.inner.remove(&key);
            self.inner.insert(key, store_val);
        } else {
            self.inner.insert(key, store_val);
        }
        Ok(())
    }
}

// Streams
// Making the decision to add a separate StreamStore to simplify the initial implementation.

const WC_STR: &str = "*";
// using the 65th bit to determine if an id is a wildcard.
const WC_BIT: u128 = 1 << 64;

#[inline]
fn is_wc(v: u128) -> bool {
    v == WC_BIT
}

#[inline]
fn get_current_time() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("?")
        .as_millis() as usize
}

// CLEAN-UP START
// Why 128 bits? To make space for a bit that can be read to determine if it's a wildcard or not.
// Is this kinda dumb? Yes
fn parse_string(id: String, seq: Option<String>) -> R<(u128, u128)> {
    let id = match &id == WC_STR {
        true => WC_BIT,
        false => {
            if let Ok(id) = id.parse::<u128>() {
                id
            } else {
                return Err(StoreError::InvalidStreamID);
            }
        }
    };

    let seq = match seq {
        Some(s) => match &s == "*" {
            true => WC_BIT,
            false => {
                if let Ok(s) = s.parse::<u128>() {
                    s
                } else {
                    return Err(StoreError::InvalidStreamID);
                }
            }
        },
        None => WC_BIT, // If we have no sequence value, assume its a wc
    };

    // If passed a value that's greater than the max possible 64 bit into and it's not a
    // wildcard, err out
    if ((id > std::usize::MAX as u128) && !is_wc(id))
        || ((seq > std::usize::MAX as u128) && !is_wc(seq))
    {
        Err(StoreError::InvalidStreamID)
    } else {
        match id == 0 && seq == 0 {
            true => Err(StoreError::StreamIDZero),
            false => Ok((id, seq)),
        }
    }
}

// IDs are a combination of an 'id' and a 'seq(uence)'
#[derive(Debug, Clone, Copy)]
pub struct StreamID {
    id: usize,
    seq: usize,
}

impl Default for StreamID {
    fn default() -> Self {
        Self {
            id: get_current_time(),
            seq: 0,
        }
    }
}

impl StreamID {
    fn new(id: usize, seq: usize) -> Self {
        Self { id, seq }
    }

    fn try_valid_stream_id(id: String, seq: Option<String>, last: Option<Self>) -> R<Self> {
        let (id, seq) = parse_string(id, seq)?;
        if let Some(l) = last {
            match (is_wc(id), is_wc(seq)) {
                (true, true) => {
                    let id = get_current_time();
                    match id == l.id {
                        true => {
                            let seq = l.seq + 1;
                            Ok(Self::new(id, seq))
                        }
                        false => Ok(Self::new(id, 0)),
                    }
                }
                (true, false) => {
                    let id = get_current_time();
                    let seq = seq as usize;
                    match id == l.id {
                        true => {
                            if seq > l.seq {
                                Ok(Self::new(id, seq))
                            } else {
                                Err(StoreError::InvalidStreamID)
                            }
                        }
                        false => Ok(Self::new(id, seq)),
                    }
                }
                (false, true) => {
                    let id = id as usize;
                    match id == l.id {
                        true => Ok(Self::new(id, l.seq + 1)),
                        false => match id > l.id {
                            true => Ok(Self::new(id, 0)),
                            false => Err(StoreError::InvalidStreamID),
                        },
                    }
                }
                (false, false) => {
                    let id = id as usize;
                    let seq = seq as usize;
                    match id == l.id {
                        true => match seq > l.seq {
                            true => Ok(Self::new(id, seq)),
                            false => Err(StoreError::InvalidStreamID),
                        },
                        false => match id > l.id {
                            true => Ok(Self::new(id, seq)),
                            false => Err(StoreError::InvalidStreamID),
                        },
                    }
                }
            }
        } else {
            let id = match is_wc(id) {
                true => get_current_time(),
                false => id as usize,
            };
            let seq = match is_wc(seq) {
                true => match id == 0 {
                    true => 1, // no 0-0
                    false => 0,
                },
                false => seq as usize,
            };
            Ok(Self::new(id, seq))
        }
    }

    fn checked_new(id: String, seq: Option<String>, stream: Option<&Vec<StreamValue>>) -> R<Self> {
        match stream {
            Some(s) => {
                let last = s.iter().last();
                match last {
                    Some(l) => Self::try_valid_stream_id(id, seq, Some(l.uid)),
                    None => Self::try_valid_stream_id(id, seq, None),
                }
            }
            None => Self::try_valid_stream_id(id, seq, None),
        }
    }
}
// CLEAN-UP END

// I'm not convinced this should go here
#[derive(Debug, Clone)]
pub struct StreamValue {
    uid: StreamID,
    _values: Vec<(String, String)>,
}

impl StreamValue {
    fn new(uid: StreamID, values: Vec<(String, String)>) -> Self {
        Self {
            uid,
            _values: values,
        }
    }
}

#[derive(Debug)]
pub struct StreamStore {
    pub inner: HashMap<String, Vec<StreamValue>>,
}

impl Default for StreamStore {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamStore {
    pub fn new() -> Self {
        let inner = HashMap::new();
        Self { inner }
    }

    pub fn try_read(&self, key: String) -> Option<&Vec<StreamValue>> {
        self.inner.get(&key)
    }

    pub fn try_write(
        &mut self,
        key: String,
        values: Vec<(String, String)>,
        stream_id: Option<(String, Option<String>)>,
    ) -> R<(usize, usize)> {
        let get = self.inner.get(&key).cloned();
        let added_id = if let Some(mut get) = get {
            let uid = match stream_id {
                Some(id) => StreamID::checked_new(id.0, id.1, Some(&get))?,
                None => StreamID::default(),
            };
            let stream_value = StreamValue::new(uid, values);
            get.push_back(stream_value);
            self.inner.remove(&key);
            self.inner.insert(key, get);
            (uid.id, uid.seq)
        } else {
            let uid = match stream_id {
                Some(id) => StreamID::checked_new(id.0, id.1, None)?,
                None => StreamID::default(),
            };
            let stream_value = StreamValue::new(uid, values);
            let stream = Vec::from([stream_value]);
            self.inner.insert(key, stream);
            (uid.id, uid.seq)
        };
        Ok(added_id)
    }
}

#[derive(Debug)]
pub struct Store {
    pub kv_store: KVStore,
    pub stream_store: StreamStore,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        let kv_store = KVStore::new();
        let stream_store = StreamStore::new();
        Self {
            kv_store,
            stream_store,
        }
    }
}
