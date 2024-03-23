use super::data::DataType;

#[derive(Debug)]
pub struct Serializer {}

impl Serializer {
    pub fn to_simple_str(str: &str) -> String {
        format!("+{}\r\n", str)
    }

    pub fn to_bulk_str(str: &str) -> String {
        format!("${}\r\n{}\r\n", str.len(), str)
    }

    pub fn to_arr(strs: Vec<&str>) -> String {
        let mut buffer = String::with_capacity(128);
        buffer.push('*');
        buffer.push_str(&strs.len().to_string());
        buffer.push_str("\r\n");
        strs.iter()
            .for_each(|s| buffer.push_str(&Self::to_bulk_str(s)));
        buffer
    }

    pub fn store_file(mut bytes: Vec<u8>) -> Vec<u8> {
        let mut buffer = format!("${}\r\n", bytes.len()).as_bytes().to_vec();
        buffer.append(&mut bytes);
        buffer
    }

    pub fn serialize_data(dt: DataType) -> String {
        match dt {
            DataType::SimpleString(s) => Serializer::to_simple_str(&s),
            DataType::BulkString(s) => Serializer::to_bulk_str(&s),
            DataType::Array(vec) => {
                let mut buff = Vec::with_capacity(8);
                for dt in vec {
                    let serial = Serializer::serialize_data(dt);
                    buff.push(serial);
                }
                // more nonsense
                Serializer::to_arr(buff.iter().map(|s| s.as_str()).collect())
            }
            _ => unimplemented!(),
        }
    }
}

mod tests {

    #[allow(unused_imports)]
    use super::Serializer;

    #[test]
    fn test_to_simple_str() {
        assert_eq!("+test\r\n", Serializer::to_simple_str("test"))
    }

    #[test]
    fn test_to_bulk_str() {
        assert_eq!("$4\r\ntest\r\n", Serializer::to_bulk_str("test"))
    }

    #[test]
    fn test_to_arr() {
        let arr = Vec::from(["test"]);
        assert_eq!("*1\r\n$4\r\ntest\r\n", Serializer::to_arr(arr))
    }
}
