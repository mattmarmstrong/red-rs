#[derive(Debug)]
pub struct Serializer {}

impl Serializer {
    pub fn to_simple_str(str: &str) -> String {
        format!("+{}\r\n", str)
    }

    pub fn to_bulk_str(str: &str) -> String {
        format!("${}\r\n{}\r\n", str.len(), str)
    }

    pub fn to_arr(strs: Vec<String>) -> String {
        let mut buffer = String::with_capacity(50);
        buffer.push('*');
        buffer.push_str(&strs.len().to_string());
        buffer.push_str("\r\n");
        strs.iter()
            .for_each(|s| buffer.push_str(&Self::to_bulk_str(s)));
        buffer
    }
}
