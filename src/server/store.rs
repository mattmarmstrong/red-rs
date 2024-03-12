use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use hashbrown::HashMap;

use super::errors::StoreError;

type R<T> = anyhow::Result<T, StoreError>;

#[derive(Debug)]
pub struct StoreValue {
    val: String,
    expiry: Option<Instant>,
}

impl StoreValue {
    pub fn new(val: String, exp: Option<Duration>) -> Self {
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

    pub fn to_val(&self) -> Option<String> {
        println!(
            "val: {}, expiry: {:#?}, now: {:#?}",
            self.val,
            self.expiry,
            Instant::now()
        );
        match self.is_expired() {
            false => Some(self.val.to_owned()),
            true => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Store {
    pub inner: Arc<RwLock<HashMap<String, StoreValue>>>,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        let inner = Arc::new(RwLock::new(HashMap::new()));
        Self { inner }
    }

    pub fn try_read(&self, key: String) -> R<Option<String>> {
        let read_lock = self.inner.read();
        match read_lock {
            Ok(store) => match store.get(&key) {
                Some(store_val) => Ok(store_val.to_val()),
                None => Ok(None),
            },
            Err(_) => Err(StoreError::ReadFailed),
        }
    }

    pub fn try_write(&self, key: String, val: String, exp: Option<Duration>) -> R<()> {
        let write_lock = self.inner.write();
        match write_lock {
            Ok(mut store) => {
                let store_val = StoreValue::new(val, exp);
                if store.get(&key).is_some() {
                    store.remove(&key);
                    store.insert(key, store_val);
                } else {
                    store.insert(key, store_val);
                }
                Ok(())
            }
            Err(_) => Err(StoreError::WriteFailed),
        }
    }
}
