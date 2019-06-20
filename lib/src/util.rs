pub mod bit_stream;
pub mod limited;

use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};

pub use crate::util::limited::Limited;
pub use crate::util::bit_stream::BitReader;

pub fn unexpected_eof_err(msg: &str) -> Error {
    Error::new(ErrorKind::UnexpectedEof, msg)
}

pub fn read_vec_limited<T: Read>(rd: &mut Limited<T>, len: usize,
    err: &'static str) -> Result<Vec<u8>>
{
    let len = len;
    if rd.max_available() < len as u64 {
        return Err(unexpected_eof_err(err));
    }
    let mut v = Vec::with_capacity(len);
    rd.take(len as u64).read_to_end(&mut v)?;
    Ok(v)
}

pub trait IoResultExt<T> {
    fn into_opt(self) -> Result<Option<T>>;
    fn map_err_eof<F, U>(self, f: impl FnOnce(Error) -> Error) -> Self;
}

impl<T> IoResultExt<T> for std::result::Result<T, Error> {
    fn into_opt(self) -> Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(e) => match e.kind() {
                ErrorKind::InvalidData | ErrorKind::UnexpectedEof => Ok(None),
                _ => Err(e),
            }
        }
    }

    fn map_err_eof<F, U>(self, f: impl FnOnce(Error) -> Error) -> Self {
        self.map_err(|e| if e.kind() == ErrorKind::UnexpectedEof {
            f(e)
        } else {
            e
        })
    }
}