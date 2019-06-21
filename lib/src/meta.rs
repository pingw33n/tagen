use std::io::prelude::*;
use std::fmt;
use std::io;
use std::time::Duration;

use crate::mpeg::Mpeg;
use crate::tags::TagsRef;
use crate::util::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormatKind {
    Mpeg,

    #[doc(hidden)]
    __Nonexhaustive,
}

#[derive(Debug)]
pub enum FormatRef<'a> {
    Mpeg(&'a Mpeg),

    #[doc(hidden)]
    __Nonexhaustive,
}

impl FormatRef<'_> {
    pub fn kind(&self) -> FormatKind {
        use FormatRef::*;
        match self {
            Mpeg(_) => FormatKind::Mpeg,
            __Nonexhaustive => unreachable!(),
        }
    }
}

impl fmt::Display for FormatRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FormatRef::*;
        match self {
            Mpeg(v) => write!(f, "MPEG {} Layer {}", v.header().version, v.header().layer),
            __Nonexhaustive => unreachable!(),
        }
    }
}

impl_enum_bits_ref!(
FormatRef:
    as_mpeg <= Mpeg ( Mpeg ),
);

enum Format {
    Mpeg(Mpeg),
}

impl Format {
    fn to_ref(&self) -> FormatRef {
        use Format::*;
        match self {
            Mpeg(v) => FormatRef::Mpeg(v),
        }
    }
}

pub struct Meta {
    format: Format,
}

impl Meta {
    pub fn read(rd: impl Read + Seek) -> io::Result<Option<Self>> {
        if let Some(f) = Mpeg::read(rd).into_opt()? {
            return Ok(Some(Self::new(Format::Mpeg(f))));
        }
        Ok(None)
    }

    pub fn format(&self) -> FormatRef {
        self.format.to_ref()
    }

    pub fn duration(&self) -> Duration {
        use Format::*;
        match &self.format {
            Mpeg(v) => v.duration(),
        }
    }

    pub fn channel_count(&self) -> u32 {
        use Format::*;
        match &self.format {
            Mpeg(v) => v.header().channel_mode.count(),
        }
    }

    pub fn samples_per_sec(&self) -> u32 {
        use Format::*;
        match &self.format {
            Mpeg(v) => v.header().samples_per_sec as u32,
        }
    }

    pub fn bits_per_sec(&self) -> u32 {
        use Format::*;
        match &self.format {
            Mpeg(v) => v.bits_per_sec() as u32,
        }
    }

    pub fn bits_per_sample(&self) -> Option<u32> {
        use Format::*;
        match &self.format {
            Mpeg(_) => None,
        }
    }

    pub fn tags(&self) -> TagsRef {
        use Format::*;
        match &self.format {
            Mpeg(v) => v.tags(),
        }
    }


    fn new(format: Format) -> Self {
        Self {
            format,
        }
    }
}

