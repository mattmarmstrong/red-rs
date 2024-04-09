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

// I'm not convinced this should go here
#[derive(Debug, Clone)]
pub struct StreamValue {
    id: usize,
    seq: usize,
    _values: Vec<(String, String)>,
}

impl StreamValue {
    fn new(id: usize, seq: usize, values: Vec<(String, String)>) -> Self {
        Self {
            id,
            seq,
            _values: values,
        }
    }
}

fn new_stream_id(last: Option<(usize, usize)>) -> R<(usize, usize)> {
    // The sun will literally explode before this would overflow 64 bits
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("?")
        .as_millis() as usize;
    match last {
        Some((id, seq)) => match current_time == id {
            true => Ok((current_time, (seq + 1))),
            false => match current_time > id {
                true => Ok((current_time, 0)),
                false => Err(StoreError::InvalidStreamID),
            },
        },
        None => Ok((current_time, 0)),
    }
}

fn get_stream_id(last: Option<(usize, usize)>, next: Option<(usize, usize)>) -> R<(usize, usize)> {
    if next.is_some() && next.unwrap() == (0, 0) {
        Err(StoreError::StreamIDZero)
    } else {
        match (last.is_some(), next.is_some()) {
            (true, true) => {
                let (last_id, last_seq) = last.unwrap();
                let (next_id, next_seq) = next.unwrap();
                match (next_id == last_id, next_seq > last_seq) {
                    (true, true) => Ok((next_id, next_seq)),
                    (true, false) => Err(StoreError::InvalidStreamID),
                    (false, _) => match next_id > last_id {
                        true => Ok((next_id, next_seq)),
                        false => Err(StoreError::InvalidStreamID),
                    },
                }
            }
            (false, true) => Ok(next.unwrap()),
            _ => new_stream_id(last),
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
        stream_id: Option<(usize, usize)>,
    ) -> R<(usize, usize)> {
        let get = self.inner.get(&key).cloned();
        let (id, seq): (usize, usize);
        if let Some(mut get) = get {
            let last = get.iter().last().unwrap();
            (id, seq) = get_stream_id(Some((last.id, last.seq)), stream_id)?;
            let stream_value = StreamValue::new(id, seq, values);
            get.push_back(stream_value);
            self.inner.remove(&key);
            self.inner.insert(key, get);
        } else {
            (id, seq) = get_stream_id(None, stream_id)?;
            let stream_value = StreamValue::new(id, seq, values);
            let stream = Vec::from([stream_value]);
            self.inner.insert(key, stream);
        };
        Ok((id, seq))
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
