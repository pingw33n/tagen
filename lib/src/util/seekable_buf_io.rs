use std::cmp;
use std::convert::{TryFrom, TryInto};
use std::io::{Error, ErrorKind, Result, SeekFrom};
use std::io::prelude::*;

const DEFAULT_BUF_CAPACITY: usize = 8 * 1024;

#[derive(Clone, Copy, Debug)]
enum InnerPos {
    /// Position right after the current buf.
    AfterBuf,

    /// Seeking has been requested but the inner stream hasn't been seeked yet.
    Desynced(SeekFrom),

    /// Position from the start of the stream.
    Synced(u64),
}

impl InnerPos {
    pub fn seek(&mut self, seek: &mut impl Seek, from: SeekFrom) -> Result<u64> {
        *self = InnerPos::Desynced(match self {
            InnerPos::AfterBuf => from,
            InnerPos::Synced(pos) => Self::combine(SeekFrom::Start(*pos), from)?,
            InnerPos::Desynced(des_from) => Self::combine(*des_from, from)?,
        });
        self.ensure_pos(seek)
    }

    pub fn ensure_not_desynced(&mut self, seek: &mut impl Seek) -> Result<Option<u64>> {
        let from = match *self {
            InnerPos::AfterBuf => return Ok(None),
            InnerPos::Synced(pos) => return Ok(Some(pos)),
            InnerPos::Desynced(from) => from,
        };
        let pos = seek.seek(from)?;
        *self = InnerPos::Synced(pos);
        Ok(Some(pos))
    }

    pub fn advance(&mut self, amt: u64) {
        match self {
            InnerPos::AfterBuf => {}
            InnerPos::Synced(pos) => *pos = check_bounds(pos.checked_add(amt)),
            InnerPos::Desynced(_) => panic!("must not be in Desynced state"),
        }
    }

    fn ensure_pos(&mut self, seek: &mut impl Seek) -> Result<u64> {
        match *self {
            InnerPos::AfterBuf => *self = InnerPos::Desynced(SeekFrom::Current(0)),
            InnerPos::Synced(pos) => return Ok(pos),
            InnerPos::Desynced(from) => match from {
                SeekFrom::Start(pos) => return Ok(pos),
                SeekFrom::Current(_) | SeekFrom::End(_) => {}
            }
        }
        Ok(self.ensure_not_desynced(seek)?.unwrap())
    }

    fn combine(s1: SeekFrom, s2: SeekFrom) -> Result<SeekFrom> {
        use SeekFrom::*;
        Ok(if let Current(s2) = s2 {
            match s1 {
                Current(s1) => Current(check_bounds_err(s1.checked_add(s2))?),
                Start(s1) => {
                    Start(if s2 < 0 {
                        let s2 = (-s2) as u64;
                        if s2 > s1 {
                            return Err(Error::new(ErrorKind::InvalidInput,
                                "can't seek beyond start"));
                        }
                        s2 - s1
                    } else {
                        check_bounds_err(s1.checked_add(s2 as u64))?
                    })
                }
                End(s1) => End(check_bounds_err(s1.checked_add(s2))?),
            }
        } else {
            s2
        })
    }
}

pub struct BufReader<T> {
    inner: T,
    inner_pos: InnerPos,
    buf: Box<[u8]>,
    buf_start: usize,
    buf_end: usize,
    buf_pos: SeekFrom,
}

impl<T> BufReader<T> {
    pub fn new(inner: T) -> Self {
        Self::with_capacity(inner, DEFAULT_BUF_CAPACITY)
    }

    pub fn with_capacity(inner: T, cap: usize) -> Self {
        assert!(i64::try_from(cap).is_ok());
        // TODO Optimize with unsafe.
        let mut buf = Vec::new();
        buf.reserve_exact(cap);
        buf.resize(cap, 0);

        Self {
            inner,
            inner_pos: InnerPos::AfterBuf,
            buf: buf.into_boxed_slice(),
            buf_start: 0,
            buf_end: 0,
            buf_pos: SeekFrom::Current(0),
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    fn buf_at(&self, pos: u64) -> &[u8] {
        if pos >= self.buf_pos + self.buf_start as u64
            && pos < self.buf_pos + self.buf_end as u64
        {
            let d = (pos - self.buf_pos) as usize;
            &self.buf[d + self.buf_start..d + self.buf_end]
        } else {
            &[]
        }
    }

    fn buf(&self) -> &[u8] {
        match &self.inner_pos {
            InnerPos::AfterBuf => self.buf_at(0),
            InnerPos::Synced(pos) => self.buf0(*pos),
            InnerPos::Desynced(from) => {
                match *from {
                    SeekFrom::Current(delta) => {
                        // delta is always the offset from the end of the current buffer.
                        if delta <= 0 {
                            let delta = -delta;
                            if delta <= (self.buf_end - self.buf_start) as i64 {
                                &self.buf[self.buf_start..self.buf_end - delta as usize]
                            } else {
                                &[]
                            }
                        } else {
                            &[]
                        }
                    }
                    SeekFrom::Start(pos) => self.buf0(pos),
                    SeekFrom::End(_) => &[],
                }
            }
        }
    }

    fn reset_buf(&mut self) {
        self.buf_start = 0;
        self.buf_end = 0;
    }
}

impl<T: Read + Seek> BufReader<T> {
    fn read_inner(inner: &mut T, inner_pos: &mut InnerPos, buf: &mut [u8]) -> Result<usize> {
        inner_pos.ensure_not_desynced(inner)?;
        match inner.read(buf) {
            Ok(amt) => {
                inner_pos.advance(amt as u64);
                Ok(amt)
            }
            Err(e) => Err(e),
        }
    }
}

impl<T: Read + Seek> BufRead for BufReader<T> {
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.buf().is_empty() {
            self.reset_buf();
            match Self::read_inner(&mut self.inner, &mut self.inner_pos, &mut self.buf) {
                Ok(amt) => self.len = check_bounds_err(amt.try_into().ok())?,
                Err(e) => return Err(e),
            }
        }
        Ok(self.buf())
    }

    fn consume(&mut self, amt: usize) {
        assert!(amt <= self.buf().len());
        let new_len = self.len - amt;
        let amt = check_bounds(i64::try_from(amt).ok());
        self.pos = check_bounds(self.pos.checked_add(amt));
        self.len = new_len;
    }
}

impl<T: Read + Seek> Read for BufReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        fn copy<'a>(src: &[u8], dst: &'a mut [u8]) -> (&'a mut [u8], usize) {
            if src.len() > 0 {
                let amt = cmp::min(src.len(), dst.len());
                dst[..amt].copy_from_slice(&src[..amt]);
                (&mut dst[amt..], amt)
            } else {
                (dst, 0)
            }
        }

        let (buf, amt) = copy(self.buf(), buf);
        self.consume(amt);

        if buf.is_empty() {
            return Ok(amt);
        }

        if buf.len() <= self.capacity() {
            let (_, amt2) = copy(self.fill_buf()?, buf);
            self.consume(amt2);

            Ok(amt + amt2)
        } else {
            self.reset_buf();
            Self::read_inner(&mut self.inner, &mut self.inner_pos, buf)
        }
    }
}

//      i
//      [   buf    ]
//            i
//      s     e
//       s    e
//        s   e
//

impl<T: Seek> Seek for BufReader<T> {
    fn seek(&mut self, from: SeekFrom) -> Result<u64> {
        match from {
            SeekFrom::Current(delta) => {
                let new_abs = match self.inner_pos {
                    InnerPos::Absolute(last_pos) => unimplemented!(),
                    InnerPos::Relative(inner_delta) => {
                        let new_delta = self.pos + delta;
                        let new_pos: i64 = check_bounds_err(
                            self.inner.seek(SeekFrom::Current(new_delta))?.try_into().ok())?;
                        self.pos = check_bounds(self.pos.checked_add(delta));
                        new_pos
                    }
                };
                self.inner_pos = InnerPos::Absolute(new_abs);
                Ok(new_abs as u64)
            }
            _ => unimplemented!()
        }
        self.inner_pos.seek(&mut self.inner, from)
    }
}

fn check_bounds<T>(v: Option<T>) -> T {
    v.expect("value overflow/underflow")
}

fn check_bounds_err<T>(v: Option<T>) -> Result<T> {
    v.ok_or_else(|| Error::new(ErrorKind::InvalidInput, "value overflow/underflow"))
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use super::*;
    use std::ops::{Bound, RangeBounds};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum IoEvent {
        Read(usize),
        Seek(SeekFrom),
    }

    struct TestReader {
        inner: Cursor<Vec<u8>>,
        events: Vec<IoEvent>,
    }

    impl TestReader {
        pub fn new(data: Vec<u8>) -> Self {
            Self {
                inner: Cursor::new(data),
                events: Vec::new(),
            }
        }
    }

    impl Read for TestReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            self.events.push(IoEvent::Read(buf.len()));
            self.inner.read(buf)
        }
    }

    impl Seek for TestReader {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
            self.events.push(IoEvent::Seek(pos));
            self.inner.seek(pos)
        }
    }

    fn read_vec(mut rd: impl Read, len: usize) -> Result<Vec<u8>> {
        let mut r = Vec::new();
        r.resize(len, 0);
        let read = rd.read(&mut r)?;
        r.truncate(read);
        Ok(r)
    }

    fn vec(range: impl RangeBounds<u8>) -> Vec<u8> {
        let start = match range.start_bound() {
            Bound::Included(v) => *v,
            _ => unimplemented!()
        };
        let end = match range.end_bound() {
            Bound::Included(v) => v + 1,
            Bound::Excluded(v) => *v,
            _ => unimplemented!()
        };
        (start..end).collect()
    }

    #[test]
    fn seq() {
        let mut rd = BufReader::with_capacity(TestReader::new(vec(1..=10)), 7);

        assert_eq!(read_vec(&mut rd, 1).unwrap(), vec(1..=1));
        assert_eq!(read_vec(&mut rd, 3).unwrap(), vec(2..=4));
        assert_eq!(read_vec(&mut rd, 6).unwrap(), vec(5..=10));
        assert_eq!(read_vec(&mut rd, 1).unwrap(), vec(0..0));

        use IoEvent::*;
        assert_eq!(rd.into_inner().events, vec![Read(7), Read(7), Read(7)]);
    }

    #[test]
    fn bypass() {
        let mut rd = BufReader::with_capacity(TestReader::new(vec(1..=10)), 3);

        assert_eq!(read_vec(&mut rd, 4).unwrap(), vec(1..=4));

        use IoEvent::*;
        assert_eq!(rd.into_inner().events, vec![Read(4)]);
    }

    #[test]
    fn seek_relative() {
        let mut rd = BufReader::with_capacity(TestReader::new(vec(1..=10)), 4);

        assert_eq!(read_vec(&mut rd, 1).unwrap(), vec(1..=1));
        assert_eq!(rd.seek(SeekFrom::Current(1)).unwrap(), 2);
        assert_eq!(read_vec(&mut rd, 1).unwrap(), vec(3..=3));

        use IoEvent::*;
        assert_eq!(rd.into_inner().events, vec![Read(4), Seek(SeekFrom::Current(-2))]);
    }
}