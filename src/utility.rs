use std::fs::File;
use std::io::Read;

use crate::result::Result;

pub fn read_string(file: &mut File, bytes: usize) -> Result<String> {
    let mut buf = vec![0; bytes as usize];
    file.read_exact(&mut buf)?;
    Ok(std::str::from_utf8(&buf)?.to_string())
}

pub fn read_int(file: &mut File) -> Result<i32> {
    let mut buf = [0; 4];
    file.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}