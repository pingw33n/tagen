#![allow(dead_code)]
#![deny(non_snake_case)]
//#![deny(unused_imports)]
#![deny(unused_must_use)]

#[macro_use]
mod macros;

pub mod id3;
pub mod mpeg;
mod util;

use mpeg::Mpeg;

pub enum Format {
    Mpeg(Mpeg),
}
