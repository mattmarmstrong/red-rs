use std::time::{Duration, Instant};

use hashbrown::HashMap;

use crate::stream::store::StreamStore;

use self::errors::StoreError;

pub mod errors;
pub mod file;

type R<T> = anyhow::Result<T, StoreError>;

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

    fn to_val(&self) -> Option<&String> {
        match self.is_expired() {
            false => Some(&self.val),
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

    pub fn try_read(&self, key: String) -> Option<&String> {
        match self.inner.get(&key) {
            Some(store_val) => store_val.to_val(),
            None => None,
        }
    }

    pub fn try_write(&mut self, key: String, val: String, exp: Option<Duration>) -> R<()> {
        let store_val = KVStoreValue::new(val, exp);
        let _ = self.inner.remove(&key);
        self.inner.insert(key, store_val);
        Ok(())
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
