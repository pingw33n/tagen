#![allow(dead_code)]
#![deny(non_snake_case)]
//#![deny(unused_imports)]
#![deny(unused_must_use)]

#[macro_use]
mod macros;

pub mod error;
pub mod flac;
pub mod id3;
pub mod meta;
pub mod mpeg;
pub mod tags;
pub mod timestamp;
mod util;
mod vcomment;

