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

    pub fn store_file(bytes: Vec<u8>) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(16);
        prefix.push(b'$');
        bytes
            .len()
            .to_ne_bytes()
            .iter()
            .for_each(|byte| prefix.push(*byte));
        prefix.push(b'\r');
        prefix.push(b'\n');
        prefix.extend(bytes);
        prefix
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
