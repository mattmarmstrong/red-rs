use regex::Regex;

use super::errors::StreamError;
use super::{StreamID, R, RANGE_GT, RANGE_LT, WC_STR};

pub struct StreamIDParser {}

// NOTE: In the context of these functions, a 'None' value represents the presence of an id wildcard
impl StreamIDParser {
    fn string_to_usize_opt(v: String) -> R<Option<usize>> {
        match v.as_str() {
            WC_STR => Ok(None),
            RANGE_LT => Ok(Some(0)),
            RANGE_GT => Ok(Some(std::usize::MAX)),
            _ => match v.parse::<usize>() {
                Ok(v) => Ok(Some(v)),
                Err(_) => Err(StreamError::InvalidStreamID),
            },
        }
    }

    pub fn split_initial(id: String) -> R<(String, Option<String>)> {
        // stream key regex
        // Match examples -> *, *-*, *-123, 123-*, 123-123, +, -
        // This will also match 1x3-12x -> That gets handled by the store.
        let re = Regex::new(r"([0-9]+|\*)-([0-9]+|\*)|\*|\+|\-").unwrap();
        if re.is_match(&id) {
            match id.contains('-') && id != RANGE_LT {
                true => {
                    let mut split = id.split('-');
                    let id = split.next().unwrap().to_string();
                    let seq = split.next().unwrap().to_string();
                    Ok((id, Some(seq)))
                }
                false => Ok((id, None)),
            }
        } else {
            Err(StreamError::InvalidStreamID)
        }
    }

    pub fn convert_to_usize_opt(
        id: String,
        seq: Option<String>,
    ) -> R<(Option<usize>, Option<usize>)> {
        let id = Self::string_to_usize_opt(id)?;
        let seq = match seq {
            Some(seq) => Self::string_to_usize_opt(seq)?,
            None => None,
        };
        match id.is_some() && seq.is_some() && id.unwrap() == 0 && seq.unwrap() == 0 {
            true => Err(StreamError::StreamIDZero),
            false => Ok((id, seq)),
        }
    }

    pub fn to_stream_id(id: String, seq: Option<String>) -> R<StreamID> {
        match Self::convert_to_usize_opt(id, seq)? {
            (Some(id), Some(seq)) => Ok(StreamID::new(id, seq)),
            (Some(id), None) => Ok(StreamID::new(id, id)), // either 0-0 or MAX-MAX
            _ => panic!("Called in the wrong context!"),
        }
    }
}
