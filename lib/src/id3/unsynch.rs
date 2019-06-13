use std::io::prelude::*;
use std::io::Result;

pub fn read_u32(rd: &mut impl Read) -> Result<Option<u32>> {
    let mut b = [0; 4];
    rd.read_exact(&mut b)?;
    Ok(decode_u32(&b))
}

pub fn decode_u32(b: &[u8]) -> Option<u32> {
    Some(
        if b[0] < 0x80 { (b[0] as u32) << 21 } else { return None; } |
        if b[1] < 0x80 { (b[1] as u32) << 14 } else { return None; } |
        if b[2] < 0x80 { (b[2] as u32) <<  7 } else { return None; } |
        if b[3] < 0x80 { (b[3] as u32) <<  0 } else { return None; }
    )
}