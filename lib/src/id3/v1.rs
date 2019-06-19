use std::io::prelude::*;
use std::io::{ErrorKind, Result, SeekFrom};

use super::string::*;

const LEN: usize = 128;
const EXT_LEN: usize = 227;
const FULL_LEN: usize = LEN + EXT_LEN;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tag {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub comment: String,
    pub track: Option<u8>,
    pub genre: Option<u8>,
    pub ext: Option<ExtTag>,
}

impl Tag {
    pub fn best_title(&self) -> &str {
        self.ext.as_ref().map(|e| &e.title).unwrap_or(&self.title)
    }

    pub fn best_artist(&self) -> &str {
        self.ext.as_ref().map(|e| &e.artist).unwrap_or(&self.artist)
    }

    pub fn best_album(&self) -> &str {
        self.ext.as_ref().map(|e| &e.album).unwrap_or(&self.album)
    }

    pub fn len(&self) -> u32 {
        (if self.ext.is_some() { FULL_LEN } else { LEN }) as u32
    }

    pub(crate) fn read(mut rd: impl Read + Seek) -> Result<Option<Self>> {
        let mut data = [0; FULL_LEN];
        for &len in &[FULL_LEN, LEN] {
            if let Err(e) = rd.seek(SeekFrom::End(-(len as i64))) {
                if e.kind() == ErrorKind::InvalidInput {
                    continue;
                }
                return Err(e);
            }
            let data = &mut data[..len];
            if let Err(e) = rd.read_exact(data) {
                if e.kind() == ErrorKind::UnexpectedEof {
                    continue;
                } else {
                    return Err(e);
                }
            }
            let r = Self::decode(&data);
            if r.is_some() {
                return Ok(r);
            }
        }
        Ok(None)
    }

    fn decode(buf: &[u8]) -> Option<Self> {
        if buf.len() < LEN || &buf[..3] != b"TAG" {
            return None;
        }

        let ext = if buf[4] == b'+' {
            if buf.len() < LEN + EXT_LEN || &buf[EXT_LEN..EXT_LEN + 3] != b"TAG" {
                return None;
            }
            Some(ExtTag::decode(buf))
        } else {
            None
        };

        let buf = if ext.is_some() {
            &buf[EXT_LEN..]
        } else {
            buf
        };

        Some(Self::decode0(buf, ext))
    }

    fn decode0(buf: &[u8], ext: Option<ExtTag>) -> Self {
        let title = decode_str(&buf[3..33]);
        let artist = decode_str(&buf[33..63]);
        let album = decode_str(&buf[63..93]);
        let year = decode_str(&buf[93..97]);

        let (comment_bytes, track) = if buf[125] == 0 && buf[126] != 0 {
            (&buf[97..125], Some(buf[126]))
        }  else {
            (&buf[97..127], None)
        };
        let comment = decode_str(comment_bytes);
        let genre = if buf[127] != 255 {
            Some(buf[127])
        } else {
            None
        };

        Self {
            title,
            artist,
            album,
            year,
            comment,
            track,
            genre,
            ext,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExtTag {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub speed: Option<u8>,
    pub genre: String,
    pub start_time: String,
    pub end_time: String,
}

impl ExtTag {
    fn decode(buf: &[u8]) -> Self {
        let title = decode_str(&buf[4..64]);
        let artist = decode_str(&buf[64..124]);
        let album = decode_str(&buf[124..184]);
        let speed = if buf[184] == 0 { None } else { Some(buf[184]) };
        let genre = decode_str(&buf[185..215]);
        let start_time = decode_str(&buf[185..215]);
        let end_time = decode_str(&buf[185..215]);
        Self {
            title,
            artist,
            album,
            speed,
            genre,
            start_time,
            end_time,
        }
    }
}

fn decode_str(buf: &[u8]) -> String {
    Decoder::new(Encoding::Latin1).decode_maybe_null_terminated(buf).unwrap()
}