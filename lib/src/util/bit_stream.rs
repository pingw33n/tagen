use bit_field::BitField;
use num::Integer;

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

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }
}

impl<T: AsRef<[u8]>> BitReader<T> {
    pub fn read_bool(&mut self, bit_count: usize) -> Option<bool> {
        self.read_u32(bit_count).map(|v| v == 1)
    }

    pub fn read_u8(&mut self, bit_count: usize) -> Option<u8> {
        const BIT_LENGTH: usize = u8::BIT_LENGTH;
        assert!(bit_count <= BIT_LENGTH);

        self.read_u32(bit_count).map(|v| v as u8)
    }

    pub fn read_u16(&mut self, bit_count: usize) -> Option<u16> {
        const BIT_LENGTH: usize = u16::BIT_LENGTH;
        assert!(bit_count <= BIT_LENGTH);

        self.read_u32(bit_count).map(|v| v as u16)
    }

    pub fn read_u32(&mut self, bit_count: usize) -> Option<u32> {
        const BIT_LENGTH: usize = u32::BIT_LENGTH;
        assert!(bit_count <= BIT_LENGTH);

        if bit_count == 0 {
            return Some(0);
        }

        let buf = self.inner.as_ref();

        let next_pos = self.pos + bit_count;
        let (next_pos_byte, next_pos_bit) = next_pos.div_rem(&8);
        if next_pos > buf.len() * 8 {
            return None;
        }

        let (pos_byte, pos_bit) = self.pos.div_rem(&8);
        self.pos = next_pos;

        let bit_start = 8 - next_pos_bit;
        let bit_end = 8 - pos_bit;
        Some(if next_pos_byte > pos_byte {
            // Leftmost partial byte.
            let mut r = buf[pos_byte].get_bits(..bit_end) as u32;

            // Middle and/or rightmost full bytes.
            let d = next_pos_byte - pos_byte;
            if d > 1 {
                r = r << 8 | buf[pos_byte + 1] as u32;
            }
            if d > 2 {
                r = r << 8 | buf[pos_byte + 2] as u32;
            }
            if d > 3 {
                r = r << 8 | buf[pos_byte + 3] as u32;
            }

            // Rightmost partial byte.
            if bit_start < 8 {
                r = r << next_pos_bit | buf[next_pos_byte].get_bits(bit_start..) as u32;
            }
            r
        } else {
            buf[pos_byte].get_bits(bit_start..bit_end) as u32
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_u8() {
        let mut rd = BitReader::new(&[0b1010_0101, 0b1010_1110, 0b1100_0101]);
        assert_eq!(rd.read_u8(0), Some(0));
        assert_eq!(rd.read_u8(8), Some(0b1010_0101));
        assert_eq!(rd.read_u8(3), Some(0b101));
        assert_eq!(rd.read_u8(8), Some(0b0_1110_110));
        assert_eq!(rd.read_u8(8), None);
        assert_eq!(rd.read_u8(5), Some(0b0_0101));
    }

    #[test]
    fn read_u32() {
        let mut rd = BitReader::new(&[0b1010_1010, 0b1010_1010, 0b1010_1010, 0b1010_1010,
            0b1010_1010, 0b1010_1010, 0b1010_1010, 0b1010_1010]);
        assert_eq!(rd.read_u32(0), Some(0));
        assert_eq!(rd.read_u32(31), Some(0b1010_1010__1010_1010__1010_1010__1010_101));
        assert_eq!(rd.read_u32(26), Some(0b0__1010_1010__1010_1010__1010_1010_1));
        assert_eq!(rd.read_u32(8), None);
        assert_eq!(rd.read_u32(7), Some(0b010_1010));
    }
}