use encoding::all::{ISO_8859_1, UTF_16BE, UTF_16LE};
use encoding::{DecoderTrap, Encoding as _};

pub use crate::id3::frame::Encoding;
use crate::error::*;

pub struct Decoder {
    encoding: Encoding,
}

impl Decoder {
    pub fn new(encoding: Encoding) -> Self {
        Self {
            encoding,
        }
    }

    pub fn decode(&self, s: &[u8]) -> Result<String> {
        use Encoding::*;
        Ok(match self.encoding {
            Latin1 => ISO_8859_1.decode(s, DecoderTrap::Strict)
                .map_err(|_| Error("bad ISO-8859-1 string"))?,
            Utf16 => {
                if s.len() < 2 {
                    return Err(Error("UTF16 BOM is truncated"));
                }
                if s[0] == 0xfe && s[1] == 0xff {
                    UTF_16BE.decode(&s[2..], DecoderTrap::Strict)
                        .map_err(|_| Error("bad UTF-16BE string"))?
                } else if s[0] == 0xff && s[1] == 0xfe {
                    UTF_16LE.decode(&s[2..], DecoderTrap::Strict)
                        .map_err(|_| Error("bad UTF-16LE string"))?
                } else {
                    return Err(Error("bad BOM"));
                }
            }
            Utf16BE => UTF_16BE.decode(&s, DecoderTrap::Strict)
                .map_err(|_| Error("bad UTF-16BE string"))?,
            Utf8 => std::str::from_utf8(s)
                .map_err(|_| Error("bad UTF-8 string"))?
                .into(),
        })
    }

    pub fn decode_null_stripped(&self, s: &[u8]) -> Result<String> {
        let s = self.strip_trailing_nulls(s);
        self.decode(s)
    }

    pub fn decode_null_terminated<'a>(&self, buf: &'a [u8]) -> Result<(String, &'a [u8])> {
        self.decode_null_terminated0(buf, true)
    }

    pub fn decode_maybe_null_terminated(&self, buf: &[u8]) -> Result<String> {
        self.decode_null_terminated0(buf, false).map(|(s, _)| s)
    }

    fn decode_null_terminated0<'a>(&self, buf: &'a [u8], fail_if_no_null: bool)
        -> Result<(String, &'a [u8])>
    {
        let (buf, rest) = match self.find_null(buf) {
            Some(i) => (&buf[..i], &buf[i + self.null_len()..]),
            None => if fail_if_no_null {
                return Err(Error("bad null-terminated string"));
            } else {
                (buf, &[][..])
            }
        };
        let s = self.decode(buf)?;
        Ok((s, rest))
    }

    pub fn decode_null_delimited(&self, buf: &[u8]) -> Result<Vec<String>> {
        let mut buf = self.strip_trailing_nulls(buf);
        let mut r = Vec::new();

        while let Some(i) = self.find_null(buf) {
            let s = self.decode(&buf[..i])?;
            r.push(s);
            buf = &buf[i + 1..];
        }
        r.push(self.decode(buf)?);

        Ok(r)
    }

    fn null_len(&self) -> usize {
        use Encoding::*;
        match self.encoding {
            Latin1 | Utf8 => 1,
            Utf16 | Utf16BE => 2,
        }
    }

    fn strip_trailing_nulls<'a>(&self, s: &'a [u8]) -> &'a [u8] {
        // Strip any trailing zeros.
        let i = if self.encoding == Encoding::Utf16 || self.encoding == Encoding::Utf16BE {
            s.rchunks_exact(2)
                .enumerate()
                .find(|(_, c)| c[0] != 0 || c[1] != 0)
                .map(|(i, _)| s.len() - i * 2)
        } else {
            s.iter().rposition(|&c| c != 0).map(|i| i + 1)
        }.unwrap_or(s.len());
        &s[..i]
    }

    fn find_null(&self, s: &[u8]) -> Option<usize> {
        if self.encoding == Encoding::Utf16 || self.encoding == Encoding::Utf16BE {
            s.chunks_exact(2)
                .enumerate()
                .find(|(_, c)| c[0] == 0 && c[1] == 0)
                .map(|(i, _)| i * 2)
        } else {
            s.iter().position(|&c| c == 0)
        }
    }
}