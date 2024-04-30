pub mod errors;
pub mod parse;
pub mod serialize;
pub mod store;

use std::time::{SystemTime, UNIX_EPOCH};

use self::errors::StreamError;
use self::parse::StreamIDParser;
use self::store::Stream;

const WC_STR: &str = "*";
// Range wildcards
const RANGE_GT: &str = "+";
const RANGE_LT: &str = "-";

#[inline]
fn get_current_time() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("?")
        .as_millis() as usize
}

type R<T> = anyhow::Result<T, StreamError>;

// IDs are a combination of an 'id' and a 'seq(uence)'
#[derive(Debug, Clone, Copy, Eq)]
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

impl PartialEq for StreamID {
    fn eq(&self, other: &Self) -> bool {
        match self.id == other.id {
            true => self.seq == other.seq,
            false => false,
        }
    }
}

impl PartialOrd for StreamID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StreamID {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.eq(other) {
            true => std::cmp::Ordering::Equal,
            false => match self.id > other.id {
                true => std::cmp::Ordering::Greater,
                false => {
                    if self.id == other.id {
                        match self.seq > other.seq {
                            true => std::cmp::Ordering::Greater,
                            false => std::cmp::Ordering::Less,
                        }
                    } else {
                        std::cmp::Ordering::Less
                    }
                }
            },
        }
    }
}

impl StreamID {
    fn new(id: usize, seq: usize) -> Self {
        Self { id, seq }
    }

    fn id_and_seq_wc(last: Self) -> R<Self> {
        let id = get_current_time();
        match id == last.id {
            true => {
                let seq = last.seq + 1;
                Ok(Self::new(id, seq))
            }
            false => Ok(Self::new(id, 0)),
        }
    }

    fn id_wc(seq: usize, last: Self) -> R<Self> {
        let id = get_current_time();
        match id == last.id {
            true => match seq > last.seq {
                true => Ok(Self::new(id, seq)),
                false => Err(StreamError::InvalidStreamID),
            },
            false => Ok(Self::new(id, seq)),
        }
    }

    fn seq_wc(id: usize, last: Self) -> R<Self> {
        match id == last.id {
            true => Ok(Self::new(id, last.seq + 1)),
            false => match id > last.id {
                true => Ok(Self::new(id, 0)),
                false => Err(StreamError::InvalidStreamID),
            },
        }
    }

    fn no_wc(id: usize, seq: usize, last: Self) -> R<Self> {
        match id == last.id {
            true => match seq > last.seq {
                true => Ok(Self::new(id, seq)),
                false => Err(StreamError::InvalidStreamID),
            },
            false => match id > last.id {
                true => Ok(Self::new(id, seq)),
                false => Err(StreamError::InvalidStreamID),
            },
        }
    }

    fn no_last(id: Option<usize>, seq: Option<usize>) -> R<Self> {
        let id = match id {
            None => get_current_time(),
            Some(id) => id,
        };
        let seq = match seq {
            None => match id == 0 {
                true => 1, // no 0-0
                false => 0,
            },
            Some(seq) => seq,
        };
        Ok(Self::new(id, seq))
    }

    fn try_valid_stream_id(id: String, seq: Option<String>, last: Option<Self>) -> R<Self> {
        let (id, seq) = StreamIDParser::convert_to_usize_opt(id, seq)?;
        match last {
            Some(last) => match (id, seq) {
                (None, None) => Self::id_and_seq_wc(last),
                (None, Some(seq)) => Self::id_wc(seq, last),
                (Some(id), None) => Self::seq_wc(id, last),
                (Some(id), Some(seq)) => Self::no_wc(id, seq, last),
            },
            None => Self::no_last(id, seq),
        }
    }

    fn checked_new(id: String, seq: Option<String>, stream: Option<&Stream>) -> R<Self> {
        match stream {
            Some(s) => match s.last_key_value() {
                Some((k, _)) => Self::try_valid_stream_id(id, seq, Some(*k)),
                None => Self::try_valid_stream_id(id, seq, None),
            },
            None => Self::try_valid_stream_id(id, seq, None),
        }
    }
}
