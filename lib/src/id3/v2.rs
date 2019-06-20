use bit_field::BitField;
use byteorder::{ReadBytesExt, BigEndian};
use std::io::prelude::*;
use std::io;

use crate::error::*;
use crate::util::*;
use super::{Timestamp, Version};
use super::frame::{FrameId, Frames};
use super::unsynch;

pub(crate) enum NoTag {
    TryForward,
    Done,
}

pub(crate) const HEADER_LEN: usize = 10;

#[derive(Debug)]
pub struct Header {
    pub version: Version,
    pub unsynch: bool,
    pub extended: bool,
    pub experimental: bool,
    pub footer_present: bool,
}

impl Header {
    fn read<T: Read>(rd: &mut Limited<T>, bytes: &[u8; HEADER_LEN], len: u32)
        -> io::Result<(Self, u32, u32)>
    {
        let version = Version::new(2, bytes[3], bytes[4]);
        if version.minor < 2 || version.minor > 4 {
            return Err(Error("bad version").into_invalid_data_err());
        }

        let flags = bytes[5];

        match version.minor {
            2 if flags & 0b0011_1111 != 0 =>
                return Err(Error("invalid flags for v2.2").into_invalid_data_err()),
            3 if flags & 0b0001_1111 != 0 =>
                return Err(Error("invalid flags for v2.3").into_invalid_data_err()),
            4 if flags & 0b0000_1111 != 0 =>
                return Err(Error("invalid flags for v2.4").into_invalid_data_err()),
            _ => {},

        }

        let unsynch = flags.get_bit(7);

        let (extended, compression) = {
            let v = flags.get_bit(6);
            // 2.2 uses this bit for compression flag.
            if version.minor == 2 {
                (false, v)
            } else {
                (v, false)
            }
        };
        if compression {
            return Err(Error("v2.2 compression is not supported").into_invalid_data_err());
        }

        let experimental = flags.get_bit(5);
        let footer_present = flags.get_bit(4);

        let ext_len = if extended {
            // FIXME https://github.com/quodlibet/quodlibet/issues/126
            let (ext_len, _data_len) = if version >= Version::V2_4 {
                let ext_len = unsynch::read_u32(rd)?
                    .ok_or_else(|| Error("extended header size is not synch safe").into_invalid_data_err())?;
                if ext_len < 4 {
                    return Err(Error("extended header size is too small").into_invalid_data_err());
                }
                (ext_len, ext_len - 4)
            } else {
                let ext_len = rd.read_u32::<BigEndian>()?;
                (ext_len, ext_len)
            };

            let _ext_data = read_vec_limited(rd, ext_len as usize, "extended header is truncated");

            ext_len
        } else {
            0
        };

        let footer_len = if footer_present { HEADER_LEN as u32 } else { 0 };
        let tag_len = HEADER_LEN as u32 + ext_len + len + footer_len;

        let header = Header {
            version,
            unsynch,
            extended,
            experimental,
            footer_present,
        };
        Ok((header, ext_len, tag_len))
    }
}

#[derive(Debug)]
pub struct Tag {
    header: Header,
    frames: Frames,
}

impl Tag {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn frames(&self) -> &Frames {
        &self.frames
    }

    pub fn album(&self) -> Option<&str> {
        self.frames.first_text_str(self.fid(FrameId::ALBUM, FrameId::V22_ALBUM))
    }

    pub fn artist(&self) -> Option<&str> {
        self.frames.first_text_str(self.fid(FrameId::ARTIST, FrameId::V22_ARTIST))
    }

    pub fn title(&self) -> Option<&str> {
        self.frames.first_text_str(self.fid(FrameId::TITLE, FrameId::V22_TITLE))
    }

    pub fn genre(&self) -> Option<&str> {
        self.frames.first_text_str(self.fid(FrameId::GENRE, FrameId::V22_GENRE))
    }

    pub fn release_date(&self) -> Option<Timestamp> {
        if self.header.version.minor == 4 {
            let s = self.frames.first_text_str(FrameId::RELEASE_DATE)?;
            s.parse().ok()
        } else {
            let year = self.frames.first_text_str(self.fid(FrameId::V23_YEAR, FrameId::V22_YEAR))
                .and_then(|s| s.parse().ok());
            if let Some(year) = year {
                let month_day = self.frames.first_text_str(self.fid(FrameId::V23_DATE, FrameId::V22_DATE))
                    .filter(|s| s.len() == 4)
                    .and_then(|s| s[..2].parse().ok().map(|m| (m, s[2..].parse().ok())));
                let month = month_day.map(|(m, _)| m);
                let day = month_day.and_then(|(_, d)| d);
                Timestamp::new(year, month, day, None, None, None)
            } else {
                None
            }
        }
    }

    pub(crate) fn read(rd: &mut impl Read, limit: Option<u64>) -> io::Result<(Self, u32)> {
        let rd = &mut Limited::new(rd, limit.unwrap_or(u64::max_value()));

        let mut bytes = [0; HEADER_LEN];
        rd.read_exact(&mut bytes)?;

        if &bytes[..3] != b"ID3"
            || bytes[4] == 0xff || bytes[5] == 0xff
        {
            return Err(Error("bad magic").into_invalid_data_err());
        }
        let len = unsynch::decode_u32(&bytes[6..10])
            .ok_or_else(|| Error("bad tag len").into_invalid_data_err())?;

        let (header, ext_len, tag_len) = Header::read(rd, &bytes, len)?;

        if header.unsynch {
            // FIXME decode
            panic!("FIXME");
        }

        let frames_len = len - ext_len;
        if rd.max_available() < frames_len as u64 {
            return Err(unexpected_eof_err("tag truncated"));
        }

        let frames = Frames::read(rd, header.version, frames_len)?;

        let tag = Self {
            header,
            frames,
        };

        Ok((tag, tag_len))
    }

    fn fid(&self, post_v2_3: FrameId, pre_v2_3: FrameId) -> FrameId {
        if self.header.version.minor >= 3 {
            post_v2_3
        } else {
            pre_v2_3
        }
    }
}