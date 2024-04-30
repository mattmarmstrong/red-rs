use std::collections::{BTreeMap, HashMap};

use super::{StreamID, R};

pub type Stream = BTreeMap<StreamID, Vec<(String, String)>>;

#[derive(Debug)]
pub struct StreamStore {
    pub inner: HashMap<String, Stream>,
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

    pub fn try_read(&self, key: String) -> Option<&Stream> {
        self.inner.get(&key)
    }

    pub fn try_write(
        &mut self,
        key: String,
        values: Vec<(String, String)>,
        stream_id: (String, Option<String>),
    ) -> R<(usize, usize)> {
        let (id, seq) = stream_id;
        let added_id = if let Some(get) = self.inner.get(&key) {
            // .get() - to borrow the stream + check the id
            let uid = StreamID::checked_new(id, seq, Some(&get))?;
            // .remove() to take ownership so it can be modified without cloning
            let mut stream = self.inner.remove(&key).unwrap();
            stream.insert(uid, values);
            self.inner.insert(key, stream);
            (uid.id, uid.seq)
        } else {
            let uid = StreamID::checked_new(id, seq, None)?;
            let mut stream = Stream::new();
            stream.insert(uid, values);
            self.inner.insert(key, stream);
            (uid.id, uid.seq)
        };
        Ok(added_id)
    }
}
