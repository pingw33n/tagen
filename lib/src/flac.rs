use bit_field::BitField;
use byteorder::{BE, ByteOrder, ReadBytesExt};
use encoding::{Encoding, DecoderTrap};
use encoding::all::{ISO_8859_1, UTF_8};
use std::cmp;
use std::convert::TryInto;
use std::io::{self, SeekFrom};
use std::io::prelude::*;

use std::time::Duration;
use std::fmt;
use crate::error::*;
use crate::util::*;
use crate::id3::v1::Id3v1;
use crate::id3::v2::Id3v2;
use crate::tags::TagsRef;
use crate::vcomment::Vcomment;

pub use crate::id3::frame::body::PictureKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamInfo {
    /// The minimum block size (in samples) used in the stream.
    pub min_block_len_samples: u16,

    /// The maximum block size (in samples) used in the stream.
    /// (Minimum blocksize == maximum blocksize) implies a fixed-blocksize stream.
    pub max_block_len_samples: u16,

    /// The minimum frame size (in bytes) used in the stream.
    pub min_frame_len_bytes: Option<u32>,

    /// The maximum frame size (in bytes) used in the stream.
    pub max_frame_len_bytes: Option<u32>,

    /// Sample rate in Hz. The maximum sample rate is limited by the structure of frame headers to
    /// 655350Hz.
    pub samples_per_sec: u32,

    /// Number of channels. FLAC supports from 1 to 8 channels
    pub channel_count: u8,

    /// Bits per sample. FLAC supports from 4 to 32 bits per sample.
    /// Currently the reference encoder and decoders only support up to 24 bits per sample.
    pub bits_per_sample: u8,

    /// Total samples in stream. 'Samples' means inter-channel sample, i.e. one second of 44.1Khz
    /// audio will have 44100 samples regardless of the number of channels.
    pub len_samples: Option<u64>,

    /// MD5 signature of the unencoded audio data.
    pub md5: [u8; 16],
}

impl StreamInfo {
    const LEN: usize = 34;

    fn read_block(mut rd: impl Read) -> io::Result<Self> {
        let hdr = BlockHeader::read(&mut rd)?;
        if hdr.len != StreamInfo::LEN as u32 {
            return Err(Error("invalid METADATA_BLOCK_STREAMINFO block len").into_invalid_data_err());
        }
        let mut buf = [0; StreamInfo::LEN];
        rd.read_exact(&mut buf)?;
        Self::decode(&buf).map_err(|e| e.into_invalid_data_err())
    }

    fn decode(buf: &[u8]) -> Result<Self> {
        if buf.len() < Self::LEN {
            return Err(Error("METADATA_BLOCK_STREAMINFO is truncated"));
        }

        let mut rd = BitReader::new(buf);

        let min_block_len_samples = rd.read_u16(16).unwrap();
        let max_block_len_samples = rd.read_u16(16).unwrap();
        let min_frame_len_bytes = Some(rd.read_u32(24).unwrap()).filter(|&v| v != 0);
        let max_frame_len_bytes = Some(rd.read_u32(24).unwrap()).filter(|&v| v != 0);

        let samples_per_sec = rd.read_u32(20).unwrap();
        if samples_per_sec == 0 {
            return Err(Error("invalid sample rate"));
        }

        let channel_count = rd.read_u8(3).unwrap() + 1;
        let bits_per_sample = rd.read_u8(5).unwrap() + 1;
        let len_samples = Some(rd.read_u64(36).unwrap()).filter(|&v| v != 0);

        let mut md5 = [0; 16];
        md5.copy_from_slice(&buf[18..]);

        Ok(Self {
            min_block_len_samples,
            max_block_len_samples,
            min_frame_len_bytes,
            max_frame_len_bytes,
            samples_per_sec,
            channel_count,
            bits_per_sample,
            len_samples,
            md5,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CueSheet {
    /// Media catalog number, in ASCII printable characters 0x20-0x7e.
    /// In general, the media catalog number may be 0 to 128 bytes long.
    /// For CD-DA, this is a thirteen digit number.
    pub media_catalog: String,

    /// The number of lead-in samples. This field has meaning only for CD-DA cuesheets;
    /// for other uses it should be 0.
    pub lead_in_sample_count: u64,

    /// `true` if the cue sheet corresponds to a Compact Disc.
    pub compact_disc: bool,

    /// One or more tracks.
    pub tracks: Vec<CueSheetTrack>,
}

impl CueSheet {
    const MIN_LEN: usize = 396;

    fn read<T: Read>(rd: &mut Limited<T>) -> io::Result<Self> {
        let mut media_catalog = [0; 128];
        rd.read_exact(&mut media_catalog)?;
        let media_catalog = decode_null_terminated_str(ISO_8859_1, &media_catalog)
            .map_err(|e| e.into_invalid_data_err())?;

        let lead_in_sample_count = rd.read_u64::<BE>()?;

        let compact_disc = rd.read_u8()?.get_bit(7);

        let mut reserved = [0; 258];
        rd.read_exact(&mut reserved)?;

        let track_count = rd.read_u8()? as usize;
        if rd.max_available() < track_count as u64 * CueSheetTrack::MIN_LEN as u64 {
            return Err(Error("METADATA_BLOCK_CUESHEET is trauncated").into_invalid_data_err());
        }
        let mut tracks = Vec::with_capacity(track_count);
        for _ in 0..track_count {
            tracks.push(CueSheetTrack::read(rd)?);
        }

        Ok(Self {
            media_catalog,
            lead_in_sample_count,
            compact_disc,
            tracks,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CueSheetTrack {
    /// Track offset in samples, relative to the beginning of the FLAC audio stream.
    pub offset_samples: u64,

    /// Track number. A track number of 0 is not allowed to avoid conflicting with the CD-DA spec,
    /// which reserves this for the lead-in. For CD-DA the number must be 1-99, or 170 for the
    /// lead-out; for non-CD-DA, the track number must for 255 for the lead-out.
    /// It is not required but encouraged to start with track 1 and increase sequentially.
    /// Track numbers must be unique within a cue sheet.
    pub num: u8,

    /// Track ISRC. This is a 12-digit alphanumeric code.
    pub isrc: Option<String>,

    /// The track type: audio or non-audio.
    pub audio: bool,

    /// The pre-emphasis flag.
    pub pre_emphasis: bool,

    pub index_entries: Vec<CueSheetIndex>,
}

impl CueSheetTrack {
    const MIN_LEN: usize = 36;

    fn read<T: Read>(rd: &mut Limited<T>) -> io::Result<Self> {
        let offset_samples = rd.read_u64::<BE>()?;
        let num = rd.read_u8()?;

        let mut isrc = [0; 12];
        rd.read_exact(&mut isrc)?;
        let isrc = Some(decode_null_terminated_str(ISO_8859_1, &isrc)
            .map_err(|e| e.into_invalid_data_err())?)
            .filter(|v| !v.is_empty());

        let flags = rd.read_u8()?;
        let audio = !flags.get_bit(7);
        let pre_emphasis = flags.get_bit(6);

        let mut reserved = [0; 13];
        rd.read_exact(&mut reserved)?;

        let index_entry_count = rd.read_u8()? as usize;
        if rd.max_available() < index_entry_count as u64 * CueSheetIndex::LEN as u64 {
            return Err(Error("CUESHEET_TRACK is truncated").into_invalid_data_err());
        }
        let mut index_entries = Vec::with_capacity(index_entry_count);
        for _ in 0..index_entry_count {
            index_entries.push(CueSheetIndex::read(rd)?);
        }

        Ok(Self {
            offset_samples,
            num,
            isrc,
            audio,
            pre_emphasis,
            index_entries,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CueSheetIndex {
    /// Offset in samples, relative to the track offset, of the index point.
    pub offset_samples: u64,

    /// The index point number.
    pub num: u8,
}

impl CueSheetIndex {
    const LEN: usize = 12;

    pub fn read(rd: &mut impl Read) -> io::Result<Self> {
        let offset_samples = rd.read_u64::<BE>()?;
        let num = rd.read_u8()?;
        let mut reserved = [0; 3];
        rd.read_exact(&mut reserved)?;
        Ok(Self {
            offset_samples,
            num,
        })
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Picture {
    pub kind: PictureKind,
    pub content_type: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub color_depth: u32,
    pub colors_used: u32,
    pub data: Vec<u8>,
}

impl Picture {
    const MIN_LEN: usize = 32;

    fn read<T: Read>(rd: &mut Limited<T>) -> io::Result<Self> {
        fn read_str<T: Read>(rd: &mut Limited<T>, encoding: &impl Encoding) -> io::Result<String> {
            let len = rd.read_u32::<BE>()?;
            let vec = read_vec_limited(rd, len as usize, "invalid string length")?;
            decode_str(encoding, &vec)
                .map_err(|e| e.into_invalid_data_err())
        }

        let kind = rd.read_u32::<BE>()?
            .try_into()
            .map_err(|_| Error("invalid picture kind").into_invalid_data_err())?;
        let kind = PictureKind(kind);

        let content_type = read_str(rd, ISO_8859_1)?;
        let description = read_str(rd, UTF_8)?;
        let width = rd.read_u32::<BE>()?;
        let height = rd.read_u32::<BE>()?;
        let color_depth = rd.read_u32::<BE>()?;
        let colors_used = rd.read_u32::<BE>()?;
        let data_len = rd.read_u32::<BE>()? as usize;
        let data = read_vec_limited(rd, data_len, "invalid data len")?;

        Ok(Self {
            kind,
            content_type,
            description,
            width,
            height,
            color_depth,
            colors_used,
            data,
        })
    }
}

impl fmt::Debug for Picture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Picture")
            .field("kind", &self.kind)
            .field("content_type", &self.content_type)
            .field("description", &self.description)
            .field("data", &display_to_debug(format!("<{} B>", self.data.len())))
            .finish()
    }
}

struct BlockHeader {
    last: bool,
    kind: BlockKind,
    len: u32,
}

impl BlockHeader {
    pub fn read(mut rd: impl Read) -> io::Result<Self> {
        let mut buf = [0; 4];
        rd.read_exact(&mut buf)?;

        let last = buf[0].get_bit(7);

        let kind = buf[0].get_bits(0..7);
        if kind == 127 {
            return Err(Error("bad block kind").into_invalid_data_err());
        }
        let kind = BlockKind(kind);

        let len = BE::read_u32(&[0, buf[1], buf[2], buf[3]]);

        Ok(Self {
            last,
            kind,
            len,
        })
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct BlockKind(u8);

impl BlockKind {
    pub const STREAM_INFO: Self = Self(0);
    pub const PADDING: Self = Self(1);
    pub const APPLICATION: Self = Self(2);
    pub const SEEKTABLE: Self = Self(3);
    pub const VORBIS_COMMENT: Self = Self(4);
    pub const CUE_SHEET: Self = Self(5);
    pub const PICTURE: Self = Self(6);
}

impl fmt::Debug for BlockKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Self::STREAM_INFO => "STREAM_INFO",
            Self::PADDING => "PADDING",
            Self::APPLICATION => "APPLICATION",
            Self::SEEKTABLE => "SEEKTABLE",
            Self::VORBIS_COMMENT => "VORBIS_COMMENT",
            Self::CUE_SHEET => "CUE_SHEET",
            Self::PICTURE => "PICTURE",
            _ => return write!(f, "BlockKind({})", self.0),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Flac {
    stream_info: StreamInfo,
    audio_len_bytes: u64,
    cue_sheet: Option<CueSheet>,
    pictures: Vec<Picture>,
    id3v1: Option<Id3v1>,
    id3v2: Option<Id3v2>,
    vcomment: Option<Vcomment>,
}

impl Flac {
    pub fn stream_info(&self) -> &StreamInfo {
        &self.stream_info
    }

    pub fn duration(&self) -> Option<Duration> {
        if let Some(l) = self.stream_info.len_samples {
            let micros = l as f64 / self.stream_info.samples_per_sec as f64 * 1_000_0000.0;
            Some(Duration::from_micros(micros as u64))
        } else {
            None
        }
    }

    pub fn bits_per_sec(&self) -> Option<u32> {
        if let Some(d) = self.duration() {
            let r = self.audio_len_bytes as f64 * 8.0 / d.as_micros() as f64 / 1_000_0000.0;
            Some(r as u32)
        } else {
            None
        }
    }

    pub fn cue_sheet(&self) -> Option<&CueSheet> {
        self.cue_sheet.as_ref()
    }

    pub fn pictures(&self) -> impl Iterator<Item=&Picture> {
        self.pictures.iter()
    }

    pub fn tags(&self) -> TagsRef {
        TagsRef {
            id3v1: self.id3v1.as_ref(),
            id3v2: self.id3v2.as_ref(),
            vcomment: self.vcomment.as_ref(),
        }
    }

    pub fn read(mut rd: impl Read + Seek) -> io::Result<Self> {
        let limit = rd.seek(SeekFrom::End(0))?;

        let id3v1 = crate::id3::v1::Id3v1::read(&mut rd).into_opt()?;

        let mut stream_info = None;
        let mut id3v2 = None;

        let pos_limit = cmp::min(1024 * 1024, limit);
        let mut pos = 0;
        while pos < pos_limit {
            rd.seek(SeekFrom::Start(pos))?;

            let mut magic = [0; 4];
            rd.read_exact(&mut magic)?;
            if &magic == b"fLaC" {
                if let Some(v) = StreamInfo::read_block(&mut rd).into_opt()? {
                    stream_info = Some(v);
                    break;
                }
            } else if id3v2.is_none() {
                rd.seek(SeekFrom::Start(pos))?;
                if let Some((id3_, id3_len)) = crate::id3::v2::Id3v2::read(&mut rd, Some(limit - pos)).into_opt()? {
                    id3v2 = Some(id3_);
                    pos += id3_len as u64;
                    continue;
                }
            }

            pos += 1;
        }
        if stream_info.is_none() {
            return Err(Error("couldn't stream info block").into_invalid_data_err());
        }

        let mut rd = Limited::new(rd, limit - pos - 4);

        let mut pictures = Vec::new();
        let mut cue_sheet = None;
        let mut vcomment = None;

        loop {
            let block_header = BlockHeader::read(&mut rd)?;

            let handled = {
                match block_header.kind {
                    BlockKind::CUE_SHEET if cue_sheet.is_none() => {
                        cue_sheet = Some(CueSheet::read(&mut rd)?);
                        true
                    }
                    BlockKind::PICTURE => {
                        pictures.push(Picture::read(&mut rd)?);
                        true
                    }
                    BlockKind::VORBIS_COMMENT if vcomment.is_none() => {
                        vcomment = Some(crate::vcomment::Vcomment::read_limited(&mut rd, false)?);
                        true
                    }
                    _ => false,
                }
            };
            // Skip if not handled.
            if !handled {
                rd.seek_relative(block_header.len as i64)?;
            }

            if block_header.last {
                break;
            }
        }

        let audio_len_bytes = rd.max_available() -
            id3v1.as_ref().map(|v| v.len() as u64).unwrap_or(0);

        Ok(Self {
            stream_info: stream_info.unwrap(),
            audio_len_bytes,
            cue_sheet,
            pictures,
            id3v1,
            id3v2,
            vcomment,
        })
    }
}

fn strip_null(s: &[u8]) -> &[u8] {
    let len = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    &s[..len]
}

fn decode_str(encoding: &impl Encoding, s: &[u8]) -> Result<String> {
    encoding.decode(s, DecoderTrap::Strict)
        .map_err(|_| Error("invalid string"))
}

fn decode_null_terminated_str(encoding: &impl Encoding, s: &[u8]) -> Result<String> {
    decode_str(encoding, strip_null(s))
}