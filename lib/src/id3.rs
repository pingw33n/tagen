pub mod frame;
pub mod string;
pub mod unsynch;
pub mod v1;
pub mod v2;

use std::cmp;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub rev: u8,
}

impl Version {
    pub const V1: Version = Self::new(1, 0, 0);
    pub const V2_2: Version = Self::new(2, 2, 0);
    pub const V2_3: Version = Self::new(2, 3, 0);
    pub const V2_4: Version = Self::new(2, 4, 0);

    pub const fn new(major: u8, minor: u8, rev: u8) -> Self {
        Self {
            major,
            minor,
            rev,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.major)?;
        if self.minor > 0 || self.rev > 0 {
            write!(f, ".{}", self.minor)?;
        }
        if self.rev > 0 {
            write!(f, ".{}", self.rev)?;
        }
        Ok(())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.major.partial_cmp(&other.major)?
            .then(self.minor.partial_cmp(&other.minor)?)
            .then(self.rev.partial_cmp(&other.rev)?))
    }
}