use crate::resp::serialize::Serializer;

use super::StreamID;

type StreamValues<'stream> = &'stream Vec<(String, String)>;
type StreamRange<'stream> = Vec<(&'stream StreamID, &'stream Vec<(String, String)>)>;

pub struct StreamSerializer {}

impl StreamSerializer {
    fn stream_id(stream_id: &StreamID) -> String {
        let s = format!("{}-{}", stream_id.id, stream_id.seq);
        Serializer::to_bulk_str(s.as_str())
    }

    fn v_to_arr(v: StreamValues) -> String {
        let mut buffer = Vec::with_capacity(v.len());
        for (k, v) in v.iter() {
            buffer.push(k.as_str());
            buffer.push(v.as_str());
        }
        Serializer::to_arr(buffer)
    }

    pub fn to_arr(stream_range: &StreamRange) -> String {
        // extremely inefficient, allocations for days. FIXME
        let mut buffer = String::with_capacity(1024);
        buffer.push('*');
        buffer.push_str(&stream_range.len().to_string());
        buffer.push_str("\r\n");
        for (id, v) in stream_range {
            buffer.push('*');
            buffer.push_str(&2.to_string());
            buffer.push_str("\r\n");
            buffer.push_str(&Self::stream_id(id));
            buffer.push_str(&Self::v_to_arr(v));
        }
        buffer
    }
}
