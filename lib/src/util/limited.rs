use std::cmp;
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom};
use std::io::prelude::*;

#[derive(Debug)]
pub struct Limited<T> {
    inner: T,
    pos: u64,
    limit: u64,
}

impl<T> Limited<T> {
    pub fn new(inner: T, limit: u64) -> Limited<T> {
        Limited {
            inner,
            pos: 0,
            limit,
        }
    }

    pub fn pos(&self) -> u64 {
        self.pos
    }

    pub fn limit(&self) -> u64 {
        self.limit
    }

    pub fn max_available(&self) -> u64 {
        self.limit - self.pos
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

fn read_limited(buf: &mut [u8], pos: u64, limit: u64, reader: &mut Read) -> Result<usize> {
    let can_read = cmp::min(buf.len() as u64, limit - pos);
    if can_read != 0 {
        reader.read(&mut buf[..(can_read as usize)])
    } else {
        Ok(0)
    }
}

impl<T: Read> Read for Limited<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read = read_limited(buf, self.pos, self.limit, &mut self.inner)?;
        self.pos += read as u64;
        Ok(read)
    }
}

impl<T: Seek> Limited<T> {
    pub fn seek_relative(&mut self, delta: i64) -> Result<u64> {
        let new_pos = if delta < 0 {
            self.pos.checked_sub((-delta) as u64)
                .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "can't seek beyond start"))?
        } else {
            self.pos.checked_add(delta as u64)
                .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "u64 overflow"))?
        };
        if new_pos > self.limit {
            return Err(Error::new(ErrorKind::InvalidInput, "can't seek past limit"));
        }
        self.inner.seek(SeekFrom::Current(delta))?;
        self.pos = new_pos;
        Ok(self.pos)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn limits_reads() {
        let mut l = Limited::new(Cursor::new(vec![1, 2, 3]), 1);

        assert_eq!(l.pos(), 0);
        assert_eq!(l.limit(), 1);
        let mut buf = [42; 3];
        assert_eq!(l.read(&mut buf).unwrap(), 1);
        assert_eq!(buf, [1, 42, 42]);
        assert_eq!(l.pos(), 1);
        assert_eq!(l.inner.position(), 1);

        buf = [42; 3];
        assert_eq!(l.read(&mut buf).unwrap(), 0);
        assert_eq!(buf, [42, 42, 42]);
        assert_eq!(l.pos(), 1);
        assert_eq!(l.into_inner().position(), 1);
    }
}
