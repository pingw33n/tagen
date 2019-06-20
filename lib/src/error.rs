use std::fmt;
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(pub(crate) &'static str);

impl Error {
    pub(crate) fn into_io_err(self, kind: io::ErrorKind) -> io::Error {
        io::Error::new(kind, self)
    }

    pub(crate) fn into_invalid_data_err(self) -> io::Error {
        self.into_io_err(io::ErrorKind::InvalidData)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}