// Equivalent to pch.h with some additions

use sdl2::sys::{SDL_DestroyTexture, SDL_Renderer, SDL_Texture, SDL_Window};
use std::io::Read;

pub struct SdlWindowPtr(pub *mut SDL_Window);
unsafe impl Sync for SdlWindowPtr {}
unsafe impl Send for SdlWindowPtr {}
pub struct SdlRendererPtr(pub *mut SDL_Renderer);
unsafe impl Send for SdlRendererPtr {}
unsafe impl Sync for SdlRendererPtr {}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SdlTexturePtr(pub *mut SDL_Texture);

impl Drop for SdlTexturePtr {
    fn drop(&mut self) {
        unsafe {
            SDL_DestroyTexture(self.0);
        }
    }
}

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

fn decode_base_85_byte(c: u8) -> u32 {
    let mut char = c;
    if char > b'\\' {
        char -= 1;
    }
    (char - 35) as u32
}

pub fn decode_base_85(input: &str) -> Result<Vec<u8>> {
    let bytes = input.as_bytes();
    if !bytes.len().is_multiple_of(5) {
        bail!("Number of bytes isn't divisible by 5");
    }

    let mut result = Vec::with_capacity(bytes.len() / 5 * 4);

    for chunk in bytes.chunks_exact(5) {
        let d = decode_base_85_byte(chunk[0])
            + decode_base_85_byte(chunk[1]) * 85
            + decode_base_85_byte(chunk[2]) * 85 * 85
            + decode_base_85_byte(chunk[3]) * 85 * 85 * 85
            + decode_base_85_byte(chunk[4]) * 85 * 85 * 85 * 85;

        result.extend_from_slice(&d.to_le_bytes());
    }

    Ok(result)
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
