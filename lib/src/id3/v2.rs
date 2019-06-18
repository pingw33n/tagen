use bit_field::BitField;
use byteorder::{ReadBytesExt, BigEndian};
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};

use crate::util::*;
use super::Version;
use super::frame::Frames;
use super::unsynch;
use crate::id3::frame::FrameId;

pub(crate) enum NoTag {
    TryForward,
    Done,
}

#[must_use]
pub(crate) enum ReadResult {
    NoTag(NoTag),
    HeaderErr(Error),
    FramesErr {
        header: Header,
        tag_len_bytes: u32,
        err: Error,
    },
    Ok { tag: Tag, len_bytes: u32 },
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
        -> Result<(Self, u32, u32)>
    {
        let version = Version::new(2, bytes[3], bytes[4]);
        if version.minor < 2 || version.minor > 4 {
            return Err(invalid_data_err("bad version"));
        }

        let flags = bytes[5];

        match version.minor {
            2 if flags & 0b0011_1111 != 0 => return Err(invalid_data_err("invalid flags for v2.2")),
            3 if flags & 0b0001_1111 != 0 => return Err(invalid_data_err("invalid flags for v2.3")),
            4 if flags & 0b0000_1111 != 0 => return Err(invalid_data_err("invalid flags for v2.4")),
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
            return Err(invalid_data_err("v2.2 compression is not supported"));
        }

        let experimental = flags.get_bit(5);
        let footer_present = flags.get_bit(4);

        let ext_len = if extended {
            // FIXME https://github.com/quodlibet/quodlibet/issues/126
            let (ext_len, _data_len) = if version >= Version::V2_4 {
                let ext_len = unsynch::read_u32(rd)?
                    .ok_or_else(|| invalid_data_err("extended header size is not synch safe"))?;
                if ext_len < 4 {
                    return Err(invalid_data_err("extended header size is too small"));
                }
                (ext_len, ext_len - 4)
            } else {
                let ext_len = rd.read_u32::<BigEndian>()?;
                (ext_len, ext_len)
            };

            let _ext_data = read_vec_limited(rd, ext_len as usize);

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

    pub(crate) fn read(rd: &mut impl Read, limit: Option<u64>) -> ReadResult {
        let rd = &mut Limited::new(rd, limit.unwrap_or(u64::max_value()));

        let mut bytes = [0; HEADER_LEN];
        match rd.read_exact(&mut bytes) {
            Ok(()) => {}
            Err(e) => return if e.kind() == ErrorKind::UnexpectedEof {
                ReadResult::NoTag(NoTag::Done)
            } else {
                ReadResult::HeaderErr(e)
            }
        }

        if &bytes[..3] != b"ID3"
            || bytes[4] == 0xff || bytes[5] == 0xff
        {
            return ReadResult::NoTag(NoTag::TryForward);
        }
        let len = if let Some(len) = unsynch::decode_u32(&bytes[6..10]) {
            len
        } else {
            return ReadResult::NoTag(NoTag::TryForward);
        };

        let (header, ext_len, tag_len) = match Header::read(rd, &bytes, len) {
            Ok(v) => v,
            Err(e) => return ReadResult::HeaderErr(e),
        };

        if header.unsynch {
            // FIXME decode
            panic!("FIXME");
        }

        let frames_len = len - ext_len;
        if rd.max_available() < frames_len as u64 {
            return ReadResult::FramesErr {
                header,
                tag_len_bytes: tag_len,
                err: unexpected_eof_err(),
            };
        }

        let frames = match Frames::read(rd, header.version, frames_len) {
            Ok(v) => v,
            Err(err) => return ReadResult::FramesErr {
                header,
                tag_len_bytes: tag_len,
                err,
            },
        };

        ReadResult::Ok {
            tag: Tag {
                header,
                frames,
            },
            len_bytes: tag_len,
        }
    }

    fn fid(&self, post_v2_3: FrameId, pre_v2_3: FrameId) -> FrameId {
        if self.header.version.minor >= 3 {
            post_v2_3
        } else {
            pre_v2_3
        }
    }
}