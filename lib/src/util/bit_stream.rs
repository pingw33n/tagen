use bit_field::BitField;
use num::Integer;
use std::io::{Error, ErrorKind, Result};
use std::io::prelude::*;
use std::cmp;

pub struct BitReader<T> {
    inner: T,
    pos: usize,
}

impl<T> BitReader<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            pos: 0,
        }
    }
}

impl<T: AsRef<[u8]>> BitReader<T> {
    pub fn seek_relative(&mut self, bit_count: isize) -> Result<usize> {
        if bit_count < 0 {
            let bit_count = (-bit_count) as usize;
            self.pos = self.pos.checked_sub(bit_count)
                .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "can't seek beyond start"))?;
        } else {
            self.pos = self.pos.checked_add(bit_count as usize)
                .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "usize overflow"))?;
        }
        Ok(self.pos)
    }

    pub fn read_bool(&mut self, bit_count: usize) -> Result<bool> {
        self.read_u32(bit_count).map(|v| v != 0)
    }

    pub fn read_u8(&mut self, bit_count: usize) -> Result<u8> {
        const BIT_LENGTH: usize = u8::BIT_LENGTH;
        assert!(bit_count <= BIT_LENGTH);

        self.read_u64(bit_count).map(|v| v as u8)
    }

    pub fn read_u16(&mut self, bit_count: usize) -> Result<u16> {
        const BIT_LENGTH: usize = u16::BIT_LENGTH;
        assert!(bit_count <= BIT_LENGTH);

        self.read_u64(bit_count).map(|v| v as u16)
    }

    pub fn read_u32(&mut self, bit_count: usize) -> Result<u32> {
        const BIT_LENGTH: usize = u32::BIT_LENGTH;
        assert!(bit_count <= BIT_LENGTH);

        self.read_u64(bit_count).map(|v| v as u32)
    }

    pub fn read_u64(&mut self, bit_count: usize) -> Result<u64> {
        const BIT_LENGTH: usize = u64::BIT_LENGTH;
        assert!(bit_count <= BIT_LENGTH);

        if bit_count == 0 {
            return Ok(0);
        }

        let buf = self.inner.as_ref();

        let next_pos = self.pos + bit_count;
        let (next_pos_byte, next_pos_bit) = next_pos.div_rem(&8);
        if next_pos > buf.len() * 8 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "unexpected EOF"));
        }

        let (pos_byte, pos_bit) = self.pos.div_rem(&8);
        self.pos = next_pos;

        let bit_start = 8 - next_pos_bit;
        let bit_end = 8 - pos_bit;
        Ok(if next_pos_byte > pos_byte {
            // Leftmost partial byte.
            let mut r = buf[pos_byte].get_bits(..bit_end) as u64;

            // Middle and/or rightmost full bytes.
            let d = next_pos_byte - pos_byte;
            for i in 1..d {
                r = r << 8 | buf[pos_byte + i] as u64;
            }

            // Rightmost partial byte.
            if bit_start < 8 {
                r = r << next_pos_bit | buf[next_pos_byte].get_bits(bit_start..) as u64;
            }
            r
        } else {
            buf[pos_byte].get_bits(bit_start..bit_end) as u64
        })
    }
}

impl<T: AsRef<[u8]>> Read for BitReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let (pos_byte, pos_bit) = self.pos.div_rem(&8);
        if pos_bit != 0 {
            unimplemented!("using read_exact not on byte boundaries is not supported");
        }

        let src = self.inner.as_ref();

        let can_read_bytes = cmp::min(src.len() - pos_byte, buf.len());
        let buf = &mut buf[..can_read_bytes];
        let next_pos_byte = pos_byte + can_read_bytes;
        buf.copy_from_slice(&src[pos_byte..next_pos_byte]);
        self.pos = next_pos_byte * 8;

        Ok(can_read_bytes)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_bool() {
        let mut rd = BitReader::new(&[0b1010_0000]);
        assert!(rd.read_bool(1).unwrap());
        assert!(!rd.read_bool(1).unwrap());
        assert!(rd.read_bool(2).unwrap());
        assert!(!rd.read_bool(4).unwrap());
    }

    #[test]
    fn read_u8() {
        let mut rd = BitReader::new(&[0b1010_0101, 0b1010_1110, 0b1100_0101]);
        assert_eq!(rd.read_u8(0).unwrap(), 0);
        assert_eq!(rd.read_u8(8).unwrap(), 0b1010_0101);
        assert_eq!(rd.read_u8(3).unwrap(), 0b101);
        assert_eq!(rd.read_u8(8).unwrap(), 0b0_1110_110);
        assert_eq!(rd.read_u8(8).unwrap_err().kind(), ErrorKind::UnexpectedEof);
        assert_eq!(rd.read_u8(5).unwrap(), 0b0_0101);
        assert_eq!(rd.read_u8(0).unwrap(), 0);
    }

    #[test]
    fn read_u32() {
        let mut rd = BitReader::new(&[0b1010_1010, 0b1010_1010, 0b1010_1010, 0b1010_1010,
            0b1010_1010, 0b1010_1010, 0b1010_1010, 0b1010_1010]);
        assert_eq!(rd.read_u32(0).unwrap(), 0);
        assert_eq!(rd.read_u32(31).unwrap(), 0b1010_1010__1010_1010__1010_1010__1010_101);
        assert_eq!(rd.read_u32(26).unwrap(), 0b0__1010_1010__1010_1010__1010_1010_1);
        assert_eq!(rd.read_u32(8).unwrap_err().kind(), ErrorKind::UnexpectedEof);
        assert_eq!(rd.read_u32(7).unwrap(), 0b010_1010);
        assert_eq!(rd.read_u32(0).unwrap(), 0);
    }

    #[test]
    fn read_exact() {
        let mut rd = BitReader::new(&[1, 2, 3, 4, 5]);
        assert_eq!(rd.read_u32(3).unwrap(), 0);
        assert_eq!(rd.read_u32(5).unwrap(), 1);

        let mut b = [0; 10];
        rd.read_exact(&mut b[..3]).unwrap();
        assert_eq!(&b[..3], &[2, 3, 4]);

        assert_eq!(rd.read_u8(8).unwrap(), 5);
    }
}