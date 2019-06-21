mod id;
pub mod body;

use bit_field::BitField;
use byteorder::{BigEndian, ByteOrder, ReadBytesExt};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::prelude::*;

use crate::error::*;
use crate::util::*;
use super::Version;
use super::unsynch;
use body::*;

pub use id::FrameId;

const HEADER_LEN: usize = 10;
const HEADER_V2_2_LEN: usize = 6;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Language(u32);

impl Language {
    pub const fn new(v: [u8; 3]) -> Self {
        Self((v[0] as u32) << 16 |
            (v[1] as u32) << 8 |
            (v[2] as u32) << 0)
    }

    pub fn to_bytes(&self) -> [u8; 3] {
        [
            (self.0 >> 16) as u8,
            (self.0 >> 8) as u8,
            (self.0 >> 0) as u8,
        ]
    }
}

impl fmt::Debug for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.to_bytes();
        write!(f, "Language(")?;
        for &c in &bytes {
            if c >= 0x20 && c <= 0x7f {
                write!(f, "{}", c as char)?;
            } else {
                write!(f, "\\x{:x}", c)?;
            }
        }
        write!(f, ")")
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Encoding {
    Latin1,
    Utf16,
    Utf16BE,
    Utf8,
}

impl Encoding {
    pub fn common(&self, o: Self) -> Self {
        if o == Encoding::Latin1 {
            *self
        } else {
            o
        }
    }

    fn from_u8(v: u8) -> Result<Self> {
        use Encoding::*;
        Ok(match v {
            0 => Latin1,
            1 => Utf16,
            2 => Utf16BE,
            3 => Utf8,
            _ => return Err(Error("bad encoding type")),
        })
    }
}

#[derive(Debug)]
pub struct Frame {
    pub id: FrameId,
    pub body: Body,
}

impl Frame {
    fn read<R: Read>(rd: &mut Limited<R>, version: Version) -> io::Result<Option<Self>> {
        assert!(version.minor >= 2 && version.minor <= 4);
        if version.minor == 2 {
            return Self::read_v2_2(rd);
        }

        let mut buf = [0; HEADER_LEN];

        let b = rd.read_u8().into_opt()?;
        if b == None || b == Some(0) {
            // EOF or padding reached.
            return Ok(None);
        }
        buf[0] = b.unwrap();

        rd.read_exact(&mut buf[1..])?;

        let flags1 = buf[8];
        let flags2 = buf[9];

        const V2_3_FLAGS_MAP: [u8; 8] = [7, 6, 5, 5, 7, 6, 0, 0];
        const V2_4_FLAGS_MAP: [u8; 8] = [6, 5, 4, 6, 3, 2, 1, 0];
        // Verify unused bits cleared and get the right flags map.
        let flags_map = match version.minor {
            3 => {
                if flags1 & 0b0001_1111 != 0 || flags2 & 0b0001_1111 != 0 {
                    return Err(Error("bad flags for frame v2.3").into_invalid_data_err());
                }
                V2_3_FLAGS_MAP
            }
            4 => {
                if flags1 & 0b1000_1111 != 0 || flags2 & 0b1011_0000 != 0 {
                    return Err(Error("bad flags for frame v2.4").into_invalid_data_err());
                }
                V2_4_FLAGS_MAP
            }
            _ => unreachable!(),
        };

        let _tag_alter_preserve = flags1.get_bit(flags_map[0] as usize);
        let _file_alter_preserve = flags1.get_bit(flags_map[1] as usize);
        let _read_only = flags1.get_bit(flags_map[2] as usize);
        let contains_group_id = flags2.get_bit(flags_map[3] as usize);
        let compressed = flags2.get_bit(flags_map[4] as usize);
        let encrypted = flags2.get_bit(flags_map[5] as usize);
        let unsynched = flags2.get_bit(flags_map[6] as usize);
        let _data_len_indicator = flags2.get_bit(flags_map[7] as usize);

        // FIXME
        assert!(!compressed);
        assert!(!contains_group_id);
        assert!(!encrypted);
        assert!(!unsynched);

        let id = FrameId::new([buf[0], buf[1], buf[2], buf[3]]);
        let len = unsynch::decode_u32(&buf[4..8])
            .ok_or_else(|| Error("bad frame len").into_invalid_data_err())?;

        let body = Body::read(rd, id, len)?;

        Ok(Some(Self {
            id,
            body,
        }))
    }

    fn read_v2_2<R: Read>(rd: &mut Limited<R>) -> io::Result<Option<Self>> {
        let mut buf = [0; HEADER_V2_2_LEN];

        let b = rd.read_u8().into_opt()?;
        if b == None || b == Some(0) {
            // EOF or padding reached.
            return Ok(None);
        }
        buf[0] = b.unwrap();

        rd.read_exact(&mut buf[1..])?;

        let id = FrameId::new_v22([buf[0], buf[1], buf[2]]);
        let len = BigEndian::read_u32(&[0, buf[3], buf[4], buf[5]]);

        let body = Body::read(rd, id, len)?;

        Ok(Some(Self {
            id,
            body,
        }))
    }
}

#[derive(Debug)]
struct FrameKey(Frame);

impl PartialEq for FrameKey {
    fn eq(&self, o: &Self) -> bool {
        use Body::*;

        let ob = &o.0.body;
        self.0.id == o.0.id &&
            match &self.0.body {
                Comment(v) => {
                    let o = ob.as_comment().unwrap();
                    v.descr == o.descr && v.lang == o.lang
                }
                UserText(v) => v.descr == ob.as_user_text().unwrap().descr,
                UserUrl(v) => v.descr == ob.as_user_url().unwrap().descr,
                Url(v) => v == ob.as_url().unwrap(),
                | Bytes(_)
                | Text(_)
                | UniqueFileId(_)
                => true,
                __Nonexhaustive => unreachable!(),
            }
    }
}

impl Eq for FrameKey {}

impl Hash for FrameKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Body::*;

        state.write(&self.0.id.to_bytes());
        match &self.0.body {
            Comment(v) => {
                state.write(v.descr.as_bytes());
                state.write(&v.lang.to_bytes());
            }
            UserText(v) => state.write(v.descr.as_bytes()),
            UserUrl(v) => state.write(v.descr.as_bytes()),
            Url(v) => state.write(v.as_bytes()),
            | Bytes(_)
            | Text(_)
            | UniqueFileId(_)
            => {}
            __Nonexhaustive => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Frames {
    vec: Vec<FrameKey>,
}

impl Frames {
    pub(crate) fn new() -> Self {
        Self {
            vec: Vec::new(),
        }
    }

    pub fn get(&self, id: FrameId) -> impl Iterator<Item = &Frame> {
        self.vec.iter().filter(move |k| k.0.id == id).map(|k| &k.0)
    }

    pub fn first(&self, id: FrameId) -> Option<&Frame> {
        self.get(id).next()
    }

    pub fn first_text(&self, id: FrameId) -> Option<&Text> {
        self.first(id).map(|f| f.body.as_text().unwrap())
    }

    pub fn first_text_str(&self, id: FrameId) -> Option<&str> {
        self.first_text(id).map(|t| t.strings[0].as_ref())
    }

    pub(crate) fn insert(&mut self, frame: Frame) {
        let key = FrameKey(frame);
        if let Some(i) = self.vec.iter().position(|k| k == &key) {
            self.vec[i].0.body.merge_from(key.0.body);
        } else {
            self.vec.push(key);
        }
    }

    pub(crate) fn read(rd: &mut impl Read, version: Version, len: u32) -> io::Result<Self> {
        if version.minor == 4 {
            // FIXME determine_bpi
        }

        Self::read0(rd, len, version)
    }

    fn read0(rd: &mut impl Read, len: u32, version: Version) -> io::Result<Frames> {
        let rd = &mut Limited::new(rd, len as u64);
        let mut r = Frames::new();
        while rd.max_available() > 0 {
            match Frame::read(rd, version) {
                Ok(Some(frame)) => r.insert(frame),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(r)
    }
}