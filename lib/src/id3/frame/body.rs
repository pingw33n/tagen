use std::io;
use std::io::prelude::*;

use crate::error::*;
use super::*;
use super::super::string::Decoder;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyKind {
    Bytes,
    Comment,
    Text,
    UniqueFileId,
    Url,
    UserText,
    UserUrl,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Body {
    Bytes(Vec<u8>),
    Comment(Comment),
    Text(Text),
    UniqueFileId(UniqueFileId),
    Url(String),
    UserText(UserText),
    UserUrl(UserUrl),
}

impl Body {
    pub fn kind(&self) -> BodyKind {
        use Body::*;
        match self {
            Bytes(_) => BodyKind::Bytes,
            Comment(_) => BodyKind::Comment,
            Text(_) => BodyKind::Text,
            UniqueFileId(_) => BodyKind::UniqueFileId,
            Url(_) => BodyKind::Url,
            UserText(_) => BodyKind::UserText,
            UserUrl(_) => BodyKind::UserUrl,
        }
    }

    pub(in super) fn merge_from(&mut self, o: Self) {
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
                debug_assert_eq!(v.descr, o.descr);
                v.text.push_str(&o.text);
            }
            Text(v) => {
                let o = o.into_text().unwrap();
                v.encoding = v.encoding.common(o.encoding);
                v.strings.extend(o.strings)
            },
            UniqueFileId(v) => *v = o.into_unique_file_id().unwrap(),
            UserText(v) => {
                let o = o.into_user_text().unwrap();
                debug_assert_eq!(v.descr, o.descr);
                v.encoding = v.encoding.common(o.encoding);
                v.values.extend(o.values);
            },
            UserUrl(v) => {
                let o = o.into_user_url().unwrap();
                debug_assert_eq!(v.descr, o.descr);
                *v = o;
            }
            Url(v) => *v = o.into_url().unwrap(),
        }
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
    pub descr: String,
    pub values: Vec<String>,
}

impl UserText {
    fn decode(buf: &[u8]) -> Result<Self> {
        let encoding = Encoding::from_u8(buf[0])?;
        let decoder = Decoder::new(encoding);
        let (descr, buf) = decoder.decode_null_terminated(&buf[1..])?;
        let values = decoder.decode_null_delimited(buf)?;

        Ok(Self {
            encoding,
            descr,
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
    pub descr: String,
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
        let (descr, buf) = decoder.decode_null_terminated(&buf[4..])?;
        let text = decoder.decode_null_stripped(buf)?;
        Ok(Self {
            encoding,
            lang,
            descr,
            text,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserUrl {
    pub encoding: Encoding,
    pub descr: String,
    pub url: String,
}

impl UserUrl {
    fn decode(buf: &[u8]) -> Result<Self> {
        let encoding = Encoding::from_u8(buf[0])?;
        let decoder = Decoder::new(encoding);
        let (descr, buf) = decoder.decode_null_terminated(&buf[1..])?;
        let url = decoder.decode_null_stripped(buf)?;

        Ok(Self {
            encoding,
            descr,
            url,
        })
    }
}