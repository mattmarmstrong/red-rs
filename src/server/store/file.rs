use std::num::ParseIntError;

pub const EMPTY: &str = "
524544495330303131fa0972656469732
d76657205372e322e30fa0a7265646973
2d62697473c040fa056374696d65c26d0
8bc65fa08757365642d6d656dc2b0c410
00fa08616f662d62617365c000fff06e3
bfec0ff5aa2";

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect()
}

pub fn empty_store_file_bytes() -> Vec<u8> {
    hex_to_bytes(EMPTY).unwrap()
}
