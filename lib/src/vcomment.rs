use bit_field::BitField;
use byteorder::{LE, ReadBytesExt};
use std::io;
use std::io::prelude::*;

use crate::error::*;
use crate::timestamp::Timestamp;
use crate::util::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vcomment {
    vendor: String,
    entries: Vec<(String, String)>,
}

impl Vcomment {
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

    pub fn title(&self) -> Option<&str> {
        self.get("TITLE").next()
    }

    pub fn artist(&self) -> Option<&str> {
        self.get("ARTIST").next()
    }

    pub fn album(&self) -> Option<&str> {
        self.get("ALBUM").next()
    }

    pub fn genre(&self) -> Option<&str> {
        self.get("GENRE").next()
    }

    pub fn date(&self) -> Option<Timestamp> {
        self.get("DATE").next().and_then(|s| s.parse().ok())
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