use std::sync::Arc;
use std::time::{Duration, Instant};

use hashbrown::HashMap;

use tokio::sync::RwLock;

use super::errors::StoreError;

pub mod file;

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

    pub async fn try_read(&self, key: String) -> R<Option<String>> {
        match self.inner.read().await.get(&key) {
            Some(store_val) => Ok(store_val.to_val()),
            None => Ok(None),
        }
    }

    pub async fn try_write(&self, key: String, val: String, exp: Option<Duration>) -> R<()> {
        let store_val = StoreValue::new(val, exp);
        let mut write = self.inner.write().await;
        if write.get(&key).is_some() {
            write.remove(&key);
            write.insert(key, store_val);
        } else {
            write.insert(key, store_val);
        }
        Ok(())
    }
}
