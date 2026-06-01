// Equivalent to pch.h

use std::ffi::c_char;
use std::io::Read;
use std::ptr::{addr_of, null};

pub fn clamp<T: std::cmp::Ord + Copy>(n: &T, lower: &T, upper: &T) -> T {
    std::cmp::max(*lower, std::cmp::min(*n, *upper))
}

pub trait LRead: Sized {
    fn lread(reader: &mut impl Read) -> std::io::Result<Self>;
}

impl LRead for u8 {
    fn lread(reader: &mut impl Read) -> std::io::Result<Self> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl LRead for i16 {
    fn lread(reader: &mut impl Read) -> std::io::Result<Self> {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }
}

impl LRead for u32 {
    fn lread(reader: &mut impl Read) -> std::io::Result<Self> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

#[cfg(target_os = "windows")]
pub const PATH_SEPARATOR: &str = "\\";
#[cfg(not(target_os = "windows"))]
pub const PATH_SEPARATOR: &str = "/";

#[cfg(target_os = "windows")]
pub const PLATFORM_DATA_PATHS: [&str; 0] = [];

#[cfg(not(target_os = "windows"))]
pub const PLATFORM_DATA_PATHS: [&str; 2] = [
    "/usr/local/share/SpaceCadetPinball/",
    "/usr/share/SpaceCadetPinball/",
];
