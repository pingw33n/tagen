use std::io;
use std::io::prelude::*;

use crate::error::*;
use super::*;
use super::super::string::Decoder;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyKind {
    Bytes,
    Comment,
    Picture,
    Text,
    UniqueFileId,
    Url,
    UserText,
    UserUrl,

    #[doc(hidden)]
    __Nonexhaustive,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Body {
    Bytes(Vec<u8>),
    Comment(Comment),
    Picture(Picture),
    Text(Text),
    UniqueFileId(UniqueFileId),
    Url(String),
    UserText(UserText),
    UserUrl(UserUrl),

    #[doc(hidden)]
    __Nonexhaustive,
}

impl Body {
    pub fn kind(&self) -> BodyKind {
        use Body::*;
        match self {
            Bytes(_) => BodyKind::Bytes,
            Comment(_) => BodyKind::Comment,
            Picture(_) => BodyKind::Picture,
            Text(_) => BodyKind::Text,
            UniqueFileId(_) => BodyKind::UniqueFileId,
            Url(_) => BodyKind::Url,
            UserText(_) => BodyKind::UserText,
            UserUrl(_) => BodyKind::UserUrl,
            __Nonexhaustive => unreachable!(),
        }
    }

    #[must_use]
    pub(in super) fn merge_from(&mut self, o: Self) -> Option<Self> {
        use Body::*;
        assert_eq!(self.kind(), o.kind());
        match self {
            Bytes(v) => *v = o.into_bytes().unwrap(),
            Comment(v) => {
                if v.text.len() > 0 {
                    v.text.push('\n');
                }
                let o = o.into_comment().unwrap();
                debug_assert_eq!(v.lang, o.lang);
                debug_assert_eq!(v.description, o.description);
                v.text.push_str(&o.text);
            }
            Picture(v) => {
                let mut o = o.into_picture().unwrap();
                debug_assert_eq!(v.description, o.description);
                o.description.push(' ');
                return Some(Picture(o));
            }
            Text(v) => {
                let o = o.into_text().unwrap();
                v.encoding = v.encoding.common(o.encoding);
                v.strings.extend(o.strings)
            },
            UniqueFileId(v) => *v = o.into_unique_file_id().unwrap(),
            UserText(v) => {
                let o = o.into_user_text().unwrap();
                debug_assert_eq!(v.description, o.description);
                v.encoding = v.encoding.common(o.encoding);
                v.values.extend(o.values);
            },
            UserUrl(v) => {
                let o = o.into_user_url().unwrap();
                debug_assert_eq!(v.description, o.description);
                *v = o;
            }
            Url(v) => *v = o.into_url().unwrap(),
            __Nonexhaustive => unreachable!(),
        }
        None
    }

    pub(crate) fn read<T: Read>(rd: &mut Limited<T>, frame_id: FrameId, len: u32)
        -> io::Result<Self>
    {
        let bytes = read_vec_limited(rd, len as usize, "frame truncated")?;
        Self::decode(frame_id, bytes).map_err(|e| e.into_invalid_data_err())
    }

    fn decode(frame_id: FrameId, buf: Vec<u8>) -> Result<Self> {
        if buf.is_empty() {
            return Err(Error("frame body is empty"));
        }
        match frame_id {
            FrameId::PICTURE => Picture::decode(&buf).map(Body::Picture),
            FrameId::V22_PICTURE => Picture::decode_v22(&buf).map(Body::Picture),
            FrameId::COMMENT | FrameId::V22_COMMENT => Comment::decode(&buf).map(Body::Comment),
            FrameId::USER_TEXT => UserText::decode(&buf).map(Body::UserText),
            FrameId::USER_URL => UserUrl::decode(&buf).map(Body::UserUrl),
            _ if frame_id.is_text() => Text::decode(&buf).map(Body::Text),
            _ if frame_id.is_url() => Self::decode_url(&buf).map(Body::Url),
            _ => Ok(Body::Bytes(buf)),
        }
    }

    fn decode_url(buf: &[u8]) -> Result<String> {
        let url =  Decoder::new(Encoding::Latin1).decode_maybe_null_terminated(buf)?;
        Ok(url)
    }
}

impl_as_into!(
Body:
    into_bytes, as_bytes, as_bytes_mut <= Bytes ( Vec<u8> ),
    into_comment, as_comment, as_comment_mut <= Comment ( Comment ),
    into_picture, as_picture, as_picture_mut <= Picture ( Picture ),
    into_text, as_text, as_text_mut <= Text ( Text ),
    into_unique_file_id, as_unqiue_file_id, as_sunqiue_file_id_mut <= UniqueFileId ( UniqueFileId ),
    into_user_text, as_user_text, as_user_text_mut <= UserText ( UserText ),
    into_user_url, as_user_url, as_user_url_mut <= UserUrl ( UserUrl ),
    into_url, as_url, as_url_mut <= Url ( String ),
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Text {
    pub encoding: Encoding,
    pub strings: Vec<String>,
}

impl Text {
    fn decode(buf: &[u8]) -> Result<Self> {
        let encoding = Encoding::from_u8(buf[0])?;
        let strings = Decoder::new(encoding).decode_null_delimited(&buf[1..], )?;
        Ok(Self {
            encoding,
            strings,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserText {
    pub encoding: Encoding,
    pub description: String,
    pub values: Vec<String>,
}

impl UserText {
    fn decode(buf: &[u8]) -> Result<Self> {
        let encoding = Encoding::from_u8(buf[0])?;
        let decoder = Decoder::new(encoding);
        let (description, buf) = decoder.decode_null_terminated(&buf[1..])?;
        let values = decoder.decode_null_delimited(buf)?;

        Ok(Self {
            encoding,
            description,
            values,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniqueFileId {
    pub owner_id: String,
    pub id: Vec<u8>
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Comment {
    pub encoding: Encoding,
    pub lang: Language,
    pub description: String,
    pub text: String,
}

impl Comment {
    fn decode(buf: &[u8]) -> Result<Self> {
        if buf.len() < 5 {
            return Err(Error("frame truncated"));
        }
        let encoding = Encoding::from_u8(buf[0])?;
        let decoder = Decoder::new(encoding);
        let lang = Language::new([buf[1], buf[2], buf[3]]);
        let (description, buf) = decoder.decode_null_terminated(&buf[4..])?;
        let text = decoder.decode_null_stripped(buf)?;
        Ok(Self {
            encoding,
            lang,
            description,
            text,
        })
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PictureKind(pub u8);

impl PictureKind {
    pub const OTHER: Self = Self(0x00);

    /// 32x32 pixels 'file icon' (PNG only)
    pub const ICON: Self = Self(0x01);

    /// Other file icon
    pub const OTHER_ICON: Self = Self(0x02);

    /// Cover (front)
    pub const COVER_FRONT: Self = Self(0x03);

    /// Cover (back)
    pub const COVER_BACK: Self = Self(0x04);

    /// Leaflet page
    pub const LEAFLET: Self = Self(0x05);

    /// Media (e.g. label side of CD)
    pub const MEDIA: Self = Self(0x06);

    /// Lead artist/lead performer/soloist
    pub const LEAD_ARTIST: Self = Self(0x07);

    /// Artist/performer
    pub const ARTIST: Self = Self(0x08);

    /// Conductor
    pub const CONDUCTOR: Self = Self(0x09);

    /// Band/Orchestra
    pub const BAND: Self = Self(0x0A);

    /// Composer
    pub const COMPOSER: Self = Self(0x0B);

    /// Lyricist/text writer
    pub const LYRICIST: Self = Self(0x0C);

    /// Recording Location
    pub const RECORDING_LOCATION: Self = Self(0x0D);

    /// During recording
    pub const DURING_RECORDING: Self = Self(0x0E);

    /// During performance
    pub const DURING_PERFORMANCE: Self = Self(0x0F);

    /// Movie/video screen capture
    pub const SCREEN_CAPTURE: Self = Self(0x10);

    /// A bright coloured fish
    pub const BRIGHT_FISH: Self = Self(0x11);

    /// Illustration
    pub const ILLUSTRATION: Self = Self(0x12);

    /// Band/artist logotype
    pub const BAND_LOGO: Self = Self(0x13);

    /// Publisher/Studio logotype
    pub const PUBLISHER_LOGOTYPE: Self = Self(0x14);
}

impl fmt::Display for PictureKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        loop {
            return write!(f, "{}", match *self {
                Self::OTHER => "other",
                Self::ICON => "file icon",
                Self::OTHER_ICON => "other file icon",
                Self::COVER_FRONT => "cover (front)",
                Self::COVER_BACK => "cover (back)",
                Self::LEAFLET => "leaflet page",
                Self::MEDIA => "media",
                Self::LEAD_ARTIST => "lead artist/lead performer/soloist",
                Self::ARTIST => "artist/performer",
                Self::CONDUCTOR => "conductor",
                Self::BAND => "band/orchestra",
                Self::COMPOSER => "composer",
                Self::LYRICIST => "lyricist/text writer",
                Self::RECORDING_LOCATION => "recording location",
                Self::DURING_RECORDING => "during recording",
                Self::DURING_PERFORMANCE => "during performance",
                Self::SCREEN_CAPTURE => "movie/video screen capture",
                Self::BRIGHT_FISH => "a bright coloured fish",
                Self::ILLUSTRATION => "illustration",
                Self::BAND_LOGO => "band/artist logotype",
                Self::PUBLISHER_LOGOTYPE => "publisher/studio logotype",
                _ => break,
            });
        }
        write!(f, "unknown ({})", self.0)
    }
}

impl fmt::Debug for PictureKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        loop {
            return write!(f, "{}", match *self {
                Self::OTHER => "OTHER",
                Self::ICON => "ICON",
                Self::OTHER_ICON => "OTHER_ICON",
                Self::COVER_FRONT => "COVER_FRONT",
                Self::COVER_BACK => "COVER_BACK",
                Self::LEAFLET => "LEAFLET",
                Self::MEDIA => "MEDIA",
                Self::LEAD_ARTIST => "LEAD_ARTIST",
                Self::ARTIST => "ARTIST",
                Self::CONDUCTOR => "CONDUCTOR",
                Self::BAND => "BAND",
                Self::COMPOSER => "COMPOSER",
                Self::LYRICIST => "LYRICIST",
                Self::RECORDING_LOCATION => "RECORDING_LOCATION",
                Self::DURING_RECORDING => "DURING_RECORDING",
                Self::DURING_PERFORMANCE => "DURING_PERFORMANCE",
                Self::SCREEN_CAPTURE => "SCREEN_CAPTURE",
                Self::BRIGHT_FISH => "BRIGHT_FISH",
                Self::ILLUSTRATION => "ILLUSTRATION",
                Self::BAND_LOGO => "BAND_LOGO",
                Self::PUBLISHER_LOGOTYPE => "PUBLISHER_LOGOTYPE",
                _ => break,
            });
        }
        write!(f, "PictureKind({})", self.0)
    }
}

impl From<u8> for PictureKind {
    fn from(v: u8) -> Self {
        Self(v)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Picture {
    pub encoding: Encoding,
    pub content_type: String,
    pub picture_kind: PictureKind,
    pub description: String,
    pub data: Vec<u8>,
}

impl Picture {
    fn decode(buf: &[u8]) -> Result<Self> {
        if buf.len() < 6 {
            return Err(Error("frame truncated"));
        }
        let encoding = Encoding::from_u8(buf[0])?;
        let decoder = Decoder::new(encoding);
        let (content_type, buf) = Decoder::new(Encoding::Latin1).decode_null_terminated(&buf[1..])?;
        if buf.len() < 2 {
            return Err(Error("frame truncated"));
        }
        let picture_kind = buf[0].into();
        let (description, buf) = decoder.decode_null_terminated(&buf[1..])?;
        let data = buf.into();
        Ok(Self {
            encoding,
            content_type,
            picture_kind,
            description,
            data,
        })
    }

    fn decode_v22(buf: &[u8]) -> Result<Self> {
        if buf.len() < 6 {
            return Err(Error("frame truncated"));
        }
        let encoding = Encoding::from_u8(buf[0])?;
        let decoder = Decoder::new(encoding);

        let mut format = [buf[1], buf[2], buf[3]];
        format.as_mut().make_ascii_lowercase();
        let content_type = match &format {
            b"bmp" => "image/bmp".to_owned(),
            b"gif" => "image/gif".to_owned(),
            b"jpg" => "image/jpeg".to_owned(),
            b"png" => "image/png".to_owned(),
            b"tif" => "image/tiff".to_owned(),
            b"-->" => "-->".to_owned(),
            _ => Decoder::new(Encoding::Latin1).decode(&format).unwrap(),
        };

        let picture_kind = buf[4].into();
        let (description, buf) = decoder.decode_null_terminated(&buf[5..])?;
        let data = buf.into();

        Ok(Self {
            encoding,
            content_type,
            picture_kind,
            description,
            data,
        })
    }
}

impl fmt::Debug for Picture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Picture")
            .field("encoding", &self.encoding)
            .field("content_type", &self.content_type)
            .field("picture_kind", &self.picture_kind)
            .field("description", &self.description)
            .field("data", &display_to_debug(format!("<{} B>", self.data.len())))
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserUrl {
    pub encoding: Encoding,
    pub description: String,
    pub url: String,
}

impl UserUrl {
    fn decode(buf: &[u8]) -> Result<Self> {
        let encoding = Encoding::from_u8(buf[0])?;
        let decoder = Decoder::new(encoding);
        let (description, buf) = decoder.decode_null_terminated(&buf[1..])?;
        let url = decoder.decode_null_stripped(buf)?;

        Ok(Self {
            encoding,
            description,
            url,
        })
    }
}