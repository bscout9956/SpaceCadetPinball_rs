use crate::partman::{Bmp8Flags, Dat8BitBmpHeader};
use sdl2::sys::SDL_Texture;
use std::cmp::PartialEq;
use std::ffi::c_char;
use std::ptr::{null, null_mut};

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum BitmapTypes {
    None = 0,
    RawBitmap = 1,
    DibBitmap = 2,
    Spliced = 3,
}

#[derive(Copy, Clone, Debug)]
pub struct ColorRgba {
    pub color: u32,
}

const ALPHA_OFFSET: u32 = 3 * 8;
const RED_OFFSET: u32 = 2 * 8;
const GREEN_OFFSET: u32 = 1 * 8;
const BLUE_OFFSET: u32 = 0; // TODO: Was 0 * 8, did I mess it up?

impl ColorRgba {
    pub const fn black() -> Self {
        Self::color_rgba(0, 0, 0, 255)
    }
    pub const fn white() -> Self {
        Self::color_rgba(255, 255, 255, 255)
    }
    pub const fn red() -> Self {
        Self::color_rgba(255, 0, 0, 255)
    }
    pub const fn green() -> Self {
        Self::color_rgba(0, 255, 0, 255)
    }
    pub const fn blue() -> Self {
        Self::color_rgba(0, 0, 255, 255)
    }

    pub const fn color_rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            color: ((alpha as u32) << ALPHA_OFFSET
                | (red as u32) << RED_OFFSET
                | (green as u32) << GREEN_OFFSET
                | (blue as u32) << BLUE_OFFSET),
        }
    }

    pub const fn color_rgba_u32(color: u32) -> Self {
        Self { color }
    }

    pub const fn get_alpha(&self) -> u8 {
        (self.color >> ALPHA_OFFSET) as u8 & 0xFFu8
    }

    pub const fn get_red(&self) -> u8 {
        (self.color >> RED_OFFSET) as u8 & 0xFFu8
    }

    pub const fn get_green(&self) -> u8 {
        (self.color >> GREEN_OFFSET) as u8 & 0xFFu8
    }

    pub const fn get_blue(&self) -> u8 {
        (self.color >> BLUE_OFFSET) as u8 & 0xFFu8
    }

    pub fn set_alpha(&mut self, val: u8) {
        self.color =
            (self.color & (!(0xFFu8 << ALPHA_OFFSET)) as u32) | (val << ALPHA_OFFSET) as u32;
    }

    pub fn set_red(&mut self, val: u8) {
        self.color = (self.color & (!(0xFFu8 << RED_OFFSET)) as u32) | (val << RED_OFFSET) as u32;
    }

    pub fn set_green(&mut self, val: u8) {
        self.color =
            (self.color & (!(0xFFu8 << GREEN_OFFSET)) as u32 | (val << GREEN_OFFSET) as u32);
    }

    pub fn set_blue(&mut self, val: u8) {
        self.color = (self.color & (!(0xFFu8 << BLUE_OFFSET)) as u32 | (val << BLUE_OFFSET) as u32);
    }
}

#[derive(Copy, Clone)]
pub struct GdrvBitmap8 {
    pub bmp_buf_ptr_1: *mut ColorRgba,
    pub indexed_bmp_ptr: *const c_char,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub indexed_stride: i32,
    pub bitmap_type: BitmapTypes,
    pub x_position: i32,
    pub y_position: i32,
    pub resolution: u32,
    pub texture: Option<SDL_Texture>,
    pub current_palette: [ColorRgba; 256],
}

impl PartialEq for BitmapTypes {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl GdrvBitmap8 {
    pub fn new(header: &Dat8BitBmpHeader) -> Self {
        assert!(
            header.width >= 0 && header.height >= 0,
            "Negative bitmap8 dimensions"
        );
        let mut instance = Self::default();

        if header.is_flag_set(Bmp8Flags::SPLICED) {
            instance.bitmap_type = BitmapTypes::Spliced;
        } else if header.is_flag_set(Bmp8Flags::DIB_BITMAP) {
            instance.bitmap_type = BitmapTypes::DibBitmap;
        } else {
            instance.bitmap_type = BitmapTypes::RawBitmap;
        }

        instance.width = header.width as i32;
        instance.stride = header.width as i32;
        instance.indexed_stride = header.width as i32;
        instance.height = header.height as i32;
        instance.x_position = header.x_position as i32;
        instance.y_position = header.y_position as i32;
        instance.resolution = header.resolution as u32;

        let mut size_in_bytes: i32;
        if instance.bitmap_type == BitmapTypes::Spliced {
            size_in_bytes = header.size;
        } else {
            if instance.bitmap_type == BitmapTypes::RawBitmap {
                assert!(
                    instance.width % 4 == 0 || header.is_flag_set(Bmp8Flags::RAW_BMP_UNALIGNED),
                    "Wrong raw bitmap align flag"
                );
            }
            if instance.width % 4 == 0 {
                instance.indexed_stride = instance.width - instance.width % 4 + 4;
            }
            size_in_bytes = instance.height * instance.indexed_stride;
            let header_size = header.size;
            assert_eq!(size_in_bytes, header_size, "Wrong bitmap8 size");
        }

        let bmp_vec: Vec<c_char> = Vec::with_capacity(size_in_bytes as usize);
        instance.indexed_bmp_ptr = bmp_vec.as_ptr();
        let mut color = ColorRgba::color_rgba_u32((instance.height * instance.stride) as u32);
        instance.bmp_buf_ptr_1 = &raw mut color;

        instance
    }

    pub fn default() -> Self {
        Self {
            bmp_buf_ptr_1: null_mut(),
            indexed_bmp_ptr: null(),
            width: 0,
            height: 0,
            stride: 0,
            indexed_stride: 0,
            bitmap_type: BitmapTypes::None,
            x_position: 0,
            y_position: 0,
            resolution: 0,
            texture: None,
            current_palette: [ColorRgba::black(); 256],
        }
    }
}
