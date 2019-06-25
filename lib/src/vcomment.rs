use bit_field::BitField;
use byteorder::{LE, ReadBytesExt};
use std::io;
use std::io::prelude::*;

use crate::error::*;
use crate::util::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tag {
    vendor: String,
    entries: Vec<(String, String)>,
}

impl Tag {
    pub fn vendor(&self) -> &str {
        &self.vendor
    }

    pub fn entries(&self) -> impl Iterator<Item=(&str, &str)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn get<'a>(&'a self, key: &'a str) -> impl 'a + Iterator<Item=&str> {
        self.entries()
            .filter(move |(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v)
    }

    pub(crate) fn read_limited<T: Read>(rd: &mut Limited<T>, framing: bool) -> io::Result<Self> {
        fn read_str<T: Read>(rd: &mut Limited<T>) -> io::Result<String> {
            let len = rd.read_u32::<LE>()?;
            let vec = read_vec_limited(rd, len as usize, "string truncated")?;
            String::from_utf8(vec).map_err(|_| Error("invalid string").into_invalid_data_err())
        }

        let vendor = read_str(rd)?;

        let count = rd.read_u32::<LE>()?;
        if count as u64 * 4 > rd.max_available() {
            return Err(Error("vcomment entries truncated").into_invalid_data_err());
        }

        let mut entries = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let s = read_str(rd)?;
            let mut parts = s.splitn(2, '=');
            let key = parts.next().unwrap().into();
            let value = parts.next().map(|v| v.to_owned()).unwrap_or_else(|| String::new());
            entries.push((key, value));
        }

        if framing && !rd.read_u8()?.get_bit(7) {
            return Err(Error("framing bit is not set").into_invalid_data_err());
        }

        Ok(Self {
            vendor,
            entries,
        })
    }
}