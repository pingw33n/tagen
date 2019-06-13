pub mod bit_stream;
pub mod limited;

use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};

pub use crate::util::limited::Limited;

pub fn invalid_data_err<E>(error: E) -> Error
    where E: Into<Box<dyn std::error::Error+Send+Sync>>
{
    Error::new(ErrorKind::InvalidData, error)
}

pub fn unexpected_eof_err() -> Error {
    Error::new(ErrorKind::UnexpectedEof, "unexpected EOF")
}

pub fn read_vec_limited<T: Read>(rd: &mut Limited<T>, len: usize) -> Result<Vec<u8>> {
    let len = len;
    if rd.max_available() < len as u64 {
        return Err(unexpected_eof_err());
    }
    let mut v = Vec::with_capacity(len);
    rd.take(len as u64).read_to_end(&mut v)?;
    Ok(v)
}