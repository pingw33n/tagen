use bit_field::BitField;
use byteorder::{BE, ReadBytesExt};
use std::io::Cursor;
use std::io::prelude::*;

use crate::error::*;
use crate::util::*;
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Vbr {
    Xing(Box<Xing>),
    Vbri(Vbri),
}

impl Vbr {
    pub fn stream_len_bytes(&self) -> Option<u32> {
        match self {
            Vbr::Xing(v) => v.stream_len_bytes,
            Vbr::Vbri(v) => Some(v.stream_len_bytes),
        }
    }

    pub fn stream_len_frames(&self) -> Option<u32> {
        match self {
            Vbr::Xing(v) => v.stream_len_frames,
            Vbr::Vbri(v) => Some(v.stream_len_frames),
        }
    }

    pub(crate) fn read(mut rd: impl Read) -> io::Result<Self> {
        let mut tag = [0; 4];
        rd.read_exact(&mut tag)?;

        match &tag[..] {
            b"Xing" | b"Info" => Xing::read(&mut rd).map(Vbr::Xing),
            b"VBRI" => Vbri::read(&mut rd).map(Vbr::Vbri),
            _ => Err(Error("bad magic").into_invalid_data_err()),
        }
    }

    pub(crate) fn offset(header: &Header) -> u64 {
        match header.version {
            Version::V1 => match header.channel_mode {
                ChannelMode::Mono => 21,
                _ => 36,
            }
            _ => match header.channel_mode {
                ChannelMode::Mono => 13,
                _ => 21,
            }
        }
    }
}

impl_as_into!(
Vbr:
    into_xing, as_xing, as_xing_mut <= Xing ( Box<Xing> ),
    into_vbri, as_vbri, as_vbri_mut <= Vbri ( Vbri ),
);

#[derive(Clone, Debug, PartialEq)]
pub struct Xing {
    pub stream_len_frames: Option<u32>,
    pub stream_len_bytes: Option<u32>,
    pub quality: Option<u32>,
    pub lame_version: Option<LameVersion>,
    pub lame: Option<Lame>,
}

impl Xing {
    fn read(mut rd: impl Read) -> io::Result<Box<Self>> {
        let flags = rd.read_u32::<BigEndian>()?;

        let stream_len_frames = if flags.get_bit(0) {
            Some(rd.read_u32::<BigEndian>()?)
        } else {
            None
        };
        let stream_len_bytes = if flags.get_bit(1) {
            Some(rd.read_u32::<BigEndian>()?)
        } else {
            None
        };
        if flags.get_bit(2) {
            let mut toc = [0; 100];
            rd.read_exact(&mut toc)?;
        }
        let quality = if flags.get_bit(3) {
            Some(rd.read_u32::<BigEndian>()?)
        } else {
            None
        };

        let lame_version = LameVersion::read(&mut rd).into_opt()?;
        let lame = if lame_version.is_some() {
            Lame::read(&mut rd).into_opt()?
        } else {
            None
        };

        Ok(Box::new(Self {
            stream_len_frames,
            stream_len_bytes,
            quality,
            lame_version,
            lame,
        }))
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LameVersion([u8; 9]);

impl LameVersion {
    fn from_bytes(buf: [u8; 9]) -> Option<Self> {
        if buf.len() < 9 {
            return None;
        }
        if buf.starts_with(b"LAME") || buf.starts_with(b"L3.99") {
            Some(Self(buf))
        } else {
            None
        }
    }

    fn read(mut rd: impl Read) -> io::Result<Self> {
        let mut buf = [0; 9];
        rd.read_exact(&mut buf)?;
        LameVersion::from_bytes(buf)
            .ok_or_else(|| Error("bad LAME version").into_invalid_data_err())
    }
}

impl fmt::Display for LameVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &c in &self.0 {
            if c == 0 {
                break;
            }
            if c.is_ascii() {
                write!(f, "{}", c as char)?;
            } else {
                write!(f, "?")?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for LameVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LameVersion({})", self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lame {
    pub vbr_method: u8,
    pub lowpass_filter: u16,
    pub track_peak: Option<f64>,
    pub track_gain_origin: u8,
    pub track_gain_adjustment: Option<f64>,
    pub album_gain_origin: u8,
    pub album_gain_adjustment: Option<f64>,
    pub encoding_flags: u8,
    pub ath: u8,
    pub kbits_per_sec: u8,
    pub encoder_delay_start: u16,
    pub encoder_padding_end: u16,
    pub source_sample_frequency_enum: u8,
    pub unwise_setting_used: bool,
    pub stereo_mode: u8,
    pub noise_shaping: u8,
    /// Applied MP3 gain -127..127. Factor is 2 ** (mp3_gain / 4).
    pub mp3_gain: i8,
    pub surround_info: u8,
    pub preset_used: u16,
    /// Length in bytes excluding any ID3 tags.
    pub music_length: u32,
    /// CRC16 of the data specified by music_length.
    pub music_crc: u16,
    /// CRC16 of this header and everything before.
    pub header_crc: u16,
}

impl Lame {
    fn read(mut rd: impl Read) -> io::Result<Self> {
        let mut buf = [0; 27];
        rd.read_exact(&mut buf)?;
        Self::decode(&buf).map_err(|e| e.into_invalid_data_err())
    }

    fn decode(buf: &[u8]) -> Result<Self> {
        let brd = &mut BitReader::new(buf);

        let revision = brd.read_u8(4).unwrap();
        if revision != 0 {
            return Err(Error("bad LAME revision"));
        }

        let vbr_method = brd.read_u8(4).unwrap();
        let lowpass_filter = brd.read_u16(8).unwrap() * 100;

        let track_peak = brd.read_u32(32).unwrap();
        let track_peak = if track_peak == 0 {
            None
        } else {
            Some(track_peak as f64 / 2u32.pow(23) as f64)
        };

        fn read_i8<T: AsRef<[u8]>>(brd: &mut BitReader<T>, bit_count: usize) -> Option<i8> {
            assert!(bit_count > 0);
            let sign = brd.read_u8(1)?;
            let v = brd.read_u8(bit_count - 1)? as i8;
            Some(if sign == 0 { v } else { -v })
        }

        fn read_i16<T: AsRef<[u8]>>(brd: &mut BitReader<T>, bit_count: usize) -> Option<i16> {
            assert!(bit_count > 0);
            let sign = brd.read_u8(1)?;
            let v = brd.read_u16(bit_count - 1)? as i16;
            Some(if sign == 0 { v } else { -v })
        }

        fn read_i32<T: AsRef<[u8]>>(brd: &mut BitReader<T>, bit_count: usize) -> Option<i32> {
            assert!(bit_count > 0);
            let sign = brd.read_u8(1)?;
            let v = brd.read_u32(bit_count - 1)? as i32;
            Some(if sign == 0 { v } else { -v })
        }

        fn read_gain<T: AsRef<[u8]>>(brd: &mut BitReader<T>, gain_kind: u8)
            -> (u8, u8, Option<f64>)
        {
            let kind = brd.read_u8(3).unwrap();
            let origin = brd.read_u8(3).unwrap();
            let adjustment = read_i32(brd, 10).unwrap();
            let adjustment = if kind == gain_kind {
                Some(adjustment as f64 / 10.0)
            } else {
                None
            };
            (kind, origin, adjustment)
        }
        let (_track_gain_kind, track_gain_origin, track_gain_adjustment) = read_gain(brd, 1);
        let (_album_gain_kind, album_gain_origin, album_gain_adjustment) = read_gain(brd, 2);

        let encoding_flags = brd.read_u8(4).unwrap();
        let ath = brd.read_u8(4).unwrap();

        let kbits_per_sec = brd.read_u8(8).unwrap();

        let encoder_delay_start = brd.read_u16(12).unwrap();
        let encoder_padding_end = brd.read_u16(12).unwrap();

        let source_sample_frequency_enum = brd.read_u8(2).unwrap();
        let unwise_setting_used = brd.read_bool(1).unwrap();
        let stereo_mode = brd.read_u8(3).unwrap();
        let noise_shaping = brd.read_u8(2).unwrap();

        let mp3_gain = read_i8(brd, 8).unwrap();
        let _ = brd.read_u8(2).unwrap();

        let surround_info = brd.read_u8(3).unwrap();
        let preset_used = brd.read_u16(11).unwrap();
        let music_length = brd.read_u32(32).unwrap();
        let music_crc = brd.read_u16(16).unwrap();
        let header_crc = brd.read_u16(16).unwrap();

        Ok(Self {
            vbr_method,
            lowpass_filter,
            track_peak,
            track_gain_origin,
            track_gain_adjustment,
            album_gain_origin,
            album_gain_adjustment,
            encoding_flags,
            ath,
            kbits_per_sec,
            encoder_delay_start,
            encoder_padding_end,
            source_sample_frequency_enum,
            unwise_setting_used,
            stereo_mode,
            noise_shaping,
            mp3_gain,
            surround_info,
            preset_used,
            music_length,
            music_crc,
            header_crc,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vbri {
    pub version: u16,
    pub quality: u16,
    pub stream_len_bytes: u32,
    pub stream_len_frames: u32,
}

impl Vbri {
    fn read(mut rd: impl Read) -> io::Result<Self> {
        let mut buf = [0; 22];

        rd.read_exact(&mut buf)?;

        let mut brd = Cursor::new(buf);

        let version = brd.read_u16::<BE>().unwrap();
        let _delay = brd.read_u16::<BE>().unwrap(); // float16
        let quality = brd.read_u16::<BE>().unwrap();
        let stream_len_bytes = brd.read_u32::<BE>().unwrap();
        let stream_len_frames = brd.read_u32::<BE>().unwrap();

        let toc_entry_count = brd.read_u16::<BE>().unwrap();
        let _toc_scale = brd.read_u16::<BE>().unwrap();
        let toc_entry_len_bytes = brd.read_u16::<BE>().unwrap();
        if toc_entry_len_bytes != 2 && toc_entry_len_bytes != 4 {
            return Err(Error("bad VBRI TOC len").into_invalid_data_err());
        }
        let _toc_entry_len_frames = brd.read_u16::<BE>().unwrap();

        let toc_len = toc_entry_len_bytes as u32 * toc_entry_count as u32;
        let mut _toc = Vec::with_capacity(toc_len as usize);
        rd.take(toc_len as u64).read_to_end(&mut _toc)?;

        Ok(Self {
            version,
            quality,
            stream_len_bytes,
            stream_len_frames,
        })
    }
}