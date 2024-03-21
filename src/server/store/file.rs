use std::num::ParseIntError;

pub const EMPTY: &[u8] = b"\
524544495330303131fa097265646973\
2d76657205372e322e30fa0a72656469\
732d62697473c040fa056374696d65c2\
6d08bc65fa08757365642d6d656dc2b0\
c41000fa08616f662d62617365c000ff\
f06e3bfec0ff5aa2";

fn hex_to_bytes(hex: &[u8]) -> Result<Vec<u8>, ParseIntError> {
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            let str = std::str::from_utf8(&hex[i..i + 2]).unwrap();
            u8::from_str_radix(str, 16)
        })
        .collect()
}

pub fn empty_store_file_bytes() -> Vec<u8> {
    hex_to_bytes(EMPTY).unwrap()
}
