use std::io::prelude::*;
use std::fmt;
use std::io;
use std::time::Duration;

use crate::mpeg::Mpeg;
use crate::flac::Flac;
use crate::tags::TagsRef;
use crate::util::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormatKind {
    Flac,
    Mpeg,

    #[doc(hidden)]
    __Nonexhaustive,
}

#[derive(Debug)]
pub enum FormatRef<'a> {
    Flac(&'a Flac),
    Mpeg(&'a Mpeg),

    #[doc(hidden)]
    __Nonexhaustive,
}

impl FormatRef<'_> {
    pub fn kind(&self) -> FormatKind {
        use FormatRef::*;
        match self {
            Flac(_) => FormatKind::Flac,
            Mpeg(_) => FormatKind::Mpeg,
            __Nonexhaustive => unreachable!(),
        }
    }
}

impl fmt::Display for FormatRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FormatRef::*;
        match self {
            Flac(_) => write!(f, "FLAC"),
            Mpeg(v) => write!(f, "MPEG {} Layer {}", v.header().version, v.header().layer),
            __Nonexhaustive => unreachable!(),
        }
    }
}

impl_enum_bits_ref!(
FormatRef:
    as_flac <= Flac ( Flac ),
    as_mpeg <= Mpeg ( Mpeg ),
);

enum Format {
    Flac(Flac),
    Mpeg(Mpeg),
}

impl Format {
    fn to_ref(&self) -> FormatRef {
        use Format::*;
        match self {
            Flac(v) => FormatRef::Flac(v),
            Mpeg(v) => FormatRef::Mpeg(v),
        }
    }
}

pub struct Meta {
    format: Format,
}

impl Meta {
    pub fn read(mut rd: impl Read + Seek) -> io::Result<Option<Self>> {
        if let Some(f) = Mpeg::read(&mut rd).into_opt()? {
            return Ok(Some(Self::new(Format::Mpeg(f))));
        }
        if let Some(f) = Flac::read(&mut rd).into_opt()? {
            return Ok(Some(Self::new(Format::Flac(f))));
        }
        Ok(None)
    }

    pub fn format(&self) -> FormatRef {
        self.format.to_ref()
    }

    pub fn duration(&self) -> Option<Duration> {
        use Format::*;
        match &self.format {
            Flac(v) => v.duration(),
            Mpeg(v) => Some(v.duration()),
        }
    }

    pub fn channel_count(&self) -> u32 {
        use Format::*;
        match &self.format {
            Flac(v) => v.stream_info().channel_count as u32,
            Mpeg(v) => v.header().channel_mode.count(),
        }
    }

    pub fn samples_per_sec(&self) -> u32 {
        use Format::*;
        match &self.format {
            Flac(v) => v.stream_info().samples_per_sec,
            Mpeg(v) => v.header().samples_per_sec as u32,
        }
    }

    pub fn bits_per_sec(&self) -> Option<u32> {
        use Format::*;
        match &self.format {
            Flac(v) => v.bits_per_sec(),
            Mpeg(v) => Some(v.bits_per_sec() as u32),
        }
    }

    pub fn bits_per_sample(&self) -> Option<u32> {
        use Format::*;
        match &self.format {
            Flac(v) => Some(v.stream_info().bits_per_sample as u32),
            Mpeg(_) => None,
        }
    }

    pub fn tags(&self) -> TagsRef {
        use Format::*;
        match &self.format {
            Flac(v) => v.tags(),
            Mpeg(v) => v.tags(),
        }
    }


    fn new(format: Format) -> Self {
        Self {
            format,
        }
    }
}

