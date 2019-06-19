mod vbr;

use bit_field::BitField;
use byteorder::{ReadBytesExt, BigEndian};
use if_chain::if_chain;
use std::cmp;
use std::fmt;
use std::io::prelude::*;
use std::io::{ErrorKind, Result, SeekFrom};
use std::time::Duration;

use super::id3;
use crate::util::bit_stream::BitReader;
pub use vbr::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Version {
    V1,
    V2,
    V2_5,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Version::*;
        write!(f, "{}", match self {
            V1 => "1",
            V2 => "2",
            V2_5 => "2.5",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Layer {
    L1,
    L2,
    L3,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Layer::*;
        write!(f, "{}", match self {
            L1 => "I",
            L2 => "II",
            L3 => "III",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ChannelMode {
    Stereo,
    JointStereo,
    DualChannel,
    Mono,
}

impl fmt::Display for ChannelMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ChannelMode::*;
        write!(f, "{}", match self {
            Stereo => "Stereo",
            JointStereo => "Joint Stereo",
            DualChannel => "Dual Channel",
            Mono => "Mono",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Emphasis {
    None,
    E50_15,
    CcitJ17,
}

enum ReadResult<T> {
    Done,
    TryForward,
    Some(T),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Header {
    pub version: Version,
    pub layer: Layer,
    pub crc_protected: bool,
    pub padded: bool,
    /// Bitrate of the first frame in kilobits ("kilo" is 1000).
    /// For CBR this is the bitrate of the whole stream.
    pub kbits_per_sec: u16,
    pub samples_per_sec: u16,
    pub channel_mode: ChannelMode,
    pub copyrighted: bool,
    pub original: bool,
    pub emphasis: Emphasis,
}

impl Header {
    fn read(mut rd: impl Read) -> Result<ReadResult<Self>> {
        let hdr_bytes = match rd.read_u32::<BigEndian>() {
            Ok(v) => v,
            Err(e) => return if e.kind() == ErrorKind::UnexpectedEof {
                Ok(ReadResult::TryForward)
            } else {
                Err(e)
            }
        };

        // Frame sync (all bits set).
        if hdr_bytes.get_bits(21..32) != 0b111_1111_1111 {
            return Ok(ReadResult::TryForward);
        }

        let version = match hdr_bytes.get_bits(19..21) {
            0b00 => Version::V2_5,
            0b01 => return Ok(ReadResult::TryForward),
            0b10 => Version::V2,
            0b11 => Version::V1,
            _ => unreachable!(),
        };
        let layer = match hdr_bytes.get_bits(17..19) {
            0b00 => return Ok(ReadResult::TryForward),
            0b01 => Layer::L3,
            0b10 => Layer::L2,
            0b11 => Layer::L1,
            _ => unreachable!(),
        };
        let crc_protected = !hdr_bytes.get_bit(16);

        type BitrateMap = [u16; 15];
        const BIRATE_V1_L1: BitrateMap = [
            0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448];
        const BIRATE_V1_L2: BitrateMap = [
            0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384];
        const BIRATE_V1_L3: BitrateMap = [
            0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
        const BIRATE_V2_L1: BitrateMap = [
            0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256];
        const BIRATE_V2_L2_L3: BitrateMap = [
            0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];
        let bitrate =
            hdr_bytes.get_bits(12..16) as usize;
        let kbits_per_sec = match version {
            _ if bitrate == 0b1111 => return Ok(ReadResult::TryForward),
            Version::V1 => match layer {
                Layer::L1 => BIRATE_V1_L1[bitrate],
                Layer::L2 => BIRATE_V1_L2[bitrate],
                Layer::L3 => BIRATE_V1_L3[bitrate],
            }
            Version::V2 | Version::V2_5 => match layer {
                Layer::L1 => BIRATE_V2_L1[bitrate],
                Layer::L2 | Layer::L3 => BIRATE_V2_L2_L3[bitrate],
            }
        };

        const SAMPLE_RATE: [[u16; 3]; 3] = [
            [44100, 22050, 11025],
            [48000, 24000, 12000],
            [32000, 16000, 8000],
        ];
        let sample_rate = hdr_bytes.get_bits(10..12) as usize;
        if sample_rate == 0b11 {
            return Ok(ReadResult::TryForward);
        }
        let samples_per_sec = SAMPLE_RATE[sample_rate][version as usize];

        let padded = hdr_bytes.get_bit(9);

        let _private = hdr_bytes.get_bit(8);
        let channel_mode = match hdr_bytes.get_bits(6..8) {
            0b00 => ChannelMode::Stereo,
            0b01 => ChannelMode::JointStereo,
            0b10 => ChannelMode::DualChannel,
            0b11 => ChannelMode::Mono,
            _ => unreachable!(),
        };
        let copyrighted = hdr_bytes.get_bit(3);
        let original = hdr_bytes.get_bit(2);
        let emphasis = match hdr_bytes.get_bits(0..2) {
            0b00 => Emphasis::None,
            0b01 => Emphasis::E50_15,
            0b10 => return Ok(ReadResult::TryForward),
            0b11 => Emphasis::CcitJ17,
            _ => unreachable!(),
        };

        Ok(ReadResult::Some(Self {
            version,
            layer,
            crc_protected,
            padded,
            kbits_per_sec,
            samples_per_sec,
            channel_mode,
            copyrighted,
            original,
            emphasis,
        }))
    }

    pub fn samples_per_frame(&self) -> u32 {
        [
            // L1   L2      L3
            [384,   1152,   1152],  // V1
            [384,   1152,   576],   // V2
            [384,   1152,   576],   // V2_5
        ][self.version as usize][self.layer as usize]
    }

    pub fn frame_len_bytes(&self) -> u32 {
        // Samples per frame / 8
        const FRAME_LEN_FACTOR: [[u32; 3]; 3] = [
            // L1   L2      L3
            [12,    144,    144],   // V1
            [12,    144,    72],    // V2
            [12,    144,    72],    // V2_5
        ];

        // Slot length per layer.
        const SLOT_LEN: [u32; 3] = [4, 1, 1];

        let factor = FRAME_LEN_FACTOR[self.version as usize][self.layer as usize];
        (factor * self.kbits_per_sec as u32 * 1000 / self.samples_per_sec as u32 +
            self.padded as u32) * SLOT_LEN[self.layer as usize]
    }
}

#[derive(Debug)]
pub struct Mpeg {
    header: Header,
    vbr: Option<Vbr>,
    id3v1: Option<id3::v1::Tag>,
    id3v2: Option<id3::v2::Tag>,
    duration: Duration,
    bits_per_sec: u32,
}

impl Mpeg {
    pub fn read(mut rd: impl Read + Seek) -> Result<Option<Self>> {
        let file_len = rd.seek(SeekFrom::End(0))? as u64;
        rd.seek(SeekFrom::Start(0))?;
        let mut id3v2_done = false;
        let mut id3v2 = None;
        let mut pos = 0;
        let pos_limit = cmp::min(file_len, 1024 * 1024);
        let (header, header_pos) = loop {
            if pos >= pos_limit {
                return Ok(None);
            }

            if !id3v2_done {
                match id3::v2::Tag::read(&mut rd, Some(file_len - pos)) {
                    id3::v2::ReadResult::NoTag(id3::v2::NoTag::Done) => id3v2_done = true,
                    id3::v2::ReadResult::NoTag(id3::v2::NoTag::TryForward) => {},
                    | id3::v2::ReadResult::HeaderErr(err)
                    | id3::v2::ReadResult::FramesErr { err, .. }
                    => return Err(err),
                    id3::v2::ReadResult::Ok { tag, len_bytes } => {
                        pos += len_bytes as u64;
                        id3v2 = Some(tag);
                        id3v2_done = true;
                        continue;
                    }
                }
            }

            rd.seek(SeekFrom::Start(pos))?;
            match Header::read(&mut rd)? {
                ReadResult::Done => return Ok(None),
                ReadResult::TryForward => {}
                ReadResult::Some(h) => break (h, pos),
            }

            pos += 1;
            rd.seek(SeekFrom::Start(pos))?;
        };


        let id3v1 = id3::v1::Tag::read(&mut rd)?;

        let vbr = if header.layer == Layer::L3 {
            let pos = header_pos + Vbr::offset(&header);
            rd.seek(SeekFrom::Start(pos))?;
            Vbr::read(&mut rd)?
        } else {
            None
        };

        let (duration, bits_per_sec) = Self::compute_duration_and_bitrate(
            &header, header_pos, file_len, vbr.as_ref(), id3v1.as_ref());

        Ok(Some(Self {
            header,
            vbr,
            id3v1,
            id3v2,
            duration,
            bits_per_sec,
        }))
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn vbr(&self) -> Option<&Vbr> {
        self.vbr.as_ref()
    }

    pub fn id3v1(&self) -> Option<&id3::v1::Tag> {
        self.id3v1.as_ref()
    }

    pub fn id3v2(&self) -> Option<&id3::v2::Tag> {
        self.id3v2.as_ref()
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn bits_per_sec(&self) -> u32 {
        self.bits_per_sec
    }

    fn compute_duration_and_bitrate(header: &Header, header_pos: u64, file_len: u64,
        vbr: Option<&Vbr>, id3v1: Option<&id3::v1::Tag>) -> (Duration, u32)
    {
        if_chain! {
            if let Some(vbr) = &vbr;
            if let Some(stream_len_bytes) = vbr.stream_len_bytes();
            if let Some(stream_len_frames) = vbr.stream_len_frames();
            then {
                let len_samples = header.samples_per_frame() * stream_len_frames;

                // Compute audio len without LAME delay and padding.
                let audio_len_samples = if let Some(lame) = &vbr.as_xing().and_then(|x| x.lame.as_ref()) {
                    len_samples
                        .saturating_sub(lame.encoder_delay_start as u32)
                        .saturating_sub(lame.encoder_padding_end as u32)
                } else {
                    len_samples
                };

                // Exclude first frame from the audio bitstream length.
                let audio_len_bytes = stream_len_bytes.saturating_sub(header.frame_len_bytes());

                let duration_secs = audio_len_samples as f64 / header.samples_per_sec as f64;
                let bits_per_sec = (audio_len_bytes as f64 * 8.0 * header.samples_per_sec as f64
                    / len_samples as f64).ceil() as u32;
                return (Duration::from_millis((duration_secs * 1000.0).ceil() as u64), bits_per_sec);
            }
        }

        // TODO calculate average bitrate over several frames.
        let bits_per_sec = header.kbits_per_sec as u32 * 1000;

        // TODO calculate stream length as difference between the first frame start and last frame end.
        let stream_len_bytes = file_len - header_pos - id3v1.map(|v| v.len()).unwrap_or(0) as u64;

        let duration_millis = (stream_len_bytes * 8 + bits_per_sec as u64 - 1) / bits_per_sec as u64;
        (Duration::from_millis(duration_millis), bits_per_sec)
    }
}