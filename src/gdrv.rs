use sdl2::sys::SDL_Texture;

#[repr(u8)]
pub enum BitmapTypes {
    None = 0,
    RawBitmap = 1,
    DibBitmap = 2,
    Spliced = 3,
}

pub struct ColorRgba {
    pub color: u32,
}

const ALPHA_OFFSET: u32 = 3 * 8;
const RED_OFFSET: u32 = 2 * 8;
const GREEN_OFFSET: u32 = 1 * 8;
const BLUE_OFFSET: u32 = 0 * 8;

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

pub struct GdrvBitmap8 {
    pub bmp_buf_ptr_1: Vec<ColorRgba>,
    pub indexed_bmp_ptr: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub indexed_stride: i32,
    pub bitmap_type: BitmapTypes,
    pub x_position: i32,
    pub y_position: i32,
    pub resolution: u32, // TODO: Source said "unsigned", but "unsigned" what?
    pub texture: SDL_Texture,
    pub current_palette: [ColorRgba; 256],
}
