use crate::partman::{Bmp8Flags, Dat8BitBmpHeader};
use crate::state::pb_game_state::PbGameState;
use crate::state::render_state::RenderState;
use crate::utils::{SdlRendererPtr, SdlTexturePtr};
use dear_imgui_rs::Ui;
use sdl2::pixels::PixelFormatEnum;
use sdl2::sys::SDL_BlendMode::SDL_BLENDMODE_NONE;
use sdl2::sys::{
    SDL_CreateTexture, SDL_DestroyTexture, SDL_HINT_RENDER_SCALE_QUALITY, SDL_LockTexture,
    SDL_SetHint, SDL_SetTextureBlendMode, SDL_UnlockTexture,
};
use std::cmp::PartialEq;
use std::ffi::{CString, c_int, c_void};
use std::fmt::Debug;
use std::ptr::{null, null_mut};
use std::slice;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[repr(u8)]
pub enum BitmapTypes {
    None = 0,
    RawBitmap = 1,
    DibBitmap = 2,
    Spliced = 3,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ColorRgba {
    pub color: u32,
}

const ALPHA_OFFSET: u32 = 3 * 8;
const RED_OFFSET: u32 = 2 * 8;
const GREEN_OFFSET: u32 = 8;
const BLUE_OFFSET: u32 = 0;

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
            color: (alpha as u32) << ALPHA_OFFSET
                | (red as u32) << RED_OFFSET
                | (green as u32) << GREEN_OFFSET
                | (blue as u32) << BLUE_OFFSET,
        }
    }

    pub const fn color_rgba_u32(color: u32) -> Self {
        Self { color }
    }

    pub const fn get_alpha(&self) -> u8 {
        (self.color >> ALPHA_OFFSET) as u8
    }

    pub const fn get_red(&self) -> u8 {
        (self.color >> RED_OFFSET) as u8
    }

    pub const fn get_green(&self) -> u8 {
        (self.color >> GREEN_OFFSET) as u8
    }

    pub const fn get_blue(&self) -> u8 {
        (self.color >> BLUE_OFFSET) as u8
    }

    pub fn set_alpha(&mut self, val: u8) {
        self.color = (self.color & !(0xFFu32 << ALPHA_OFFSET)) | ((val as u32) << ALPHA_OFFSET);
    }

    pub fn set_red(&mut self, val: u8) {
        self.color = (self.color & !(0xFFu32 << RED_OFFSET)) | ((val as u32) << RED_OFFSET);
    }

    pub fn set_green(&mut self, val: u8) {
        self.color = (self.color & !(0xFFu32 << GREEN_OFFSET)) | ((val as u32) << GREEN_OFFSET);
    }

    pub fn set_blue(&mut self, val: u8) {
        self.color = (self.color & !(0xFFu32 << BLUE_OFFSET)) | ((val as u32) << BLUE_OFFSET);
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct GdrvBitmap8 {
    pub bmp_buffer_data: Vec<ColorRgba>,
    pub indexed_bmp_data: Vec<u8>,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub indexed_stride: i32,
    pub bitmap_type: BitmapTypes,
    pub x_position: i32,
    pub y_position: i32,
    pub resolution: u32,
    pub texture: Option<SdlTexturePtr>,
}

impl GdrvBitmap8 {
    pub unsafe fn blit_to_texture(&mut self) {
        let mut pitch = 0 as c_int;
        let mut locked_pixels_ptr: *mut c_void = null_mut();

        if let Some(tex) = self.texture.as_mut() {
            let result =
                unsafe { SDL_LockTexture(tex.0, null(), &mut locked_pixels_ptr, &mut pitch) };
            if result != 0 {
                panic!("We are updating a non-streaming texture!");
            }

            let width_bytes = (self.width as usize) * size_of::<ColorRgba>();

            if pitch as usize == width_bytes {
                let dst_slice = unsafe {
                    slice::from_raw_parts_mut(
                        locked_pixels_ptr as *mut ColorRgba,
                        self.bmp_buffer_data.len(),
                    )
                };

                dst_slice.copy_from_slice(&self.bmp_buffer_data);
            } else {
                panic!("Padding on vScreen texture");
            }

            unsafe { SDL_UnlockTexture(tex.0) };
        } else {
            panic!("We are updating a null texture");
        }
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

        let size_in_bytes: i32;
        if instance.bitmap_type == BitmapTypes::Spliced {
            size_in_bytes = header.size;
        } else {
            if instance.bitmap_type == BitmapTypes::RawBitmap {
                assert!(
                    instance.width % 4 == 0 || header.is_flag_set(Bmp8Flags::RAW_BMP_UNALIGNED),
                    "Wrong raw bitmap align flag"
                );
            }
            if instance.width % 4 != 0 {
                instance.indexed_stride = instance.width - instance.width % 4 + 4;
            }
            size_in_bytes = instance.height * instance.indexed_stride;
            let header_size = header.size;
            assert_eq!(size_in_bytes, header_size, "Wrong bitmap8 size");
        }

        let bmp_vec: Vec<u8> = vec![0; size_in_bytes as usize];
        instance.indexed_bmp_data = bmp_vec;
        instance.bmp_buffer_data =
            vec![ColorRgba::black(); (instance.height * instance.stride) as usize];

        instance
    }

    pub fn new_dims(width: i32, height: i32) -> Self {
        Self::new_dims_indexed_buff(width, height, true, true)
    }

    pub fn new_dims_indexed(width: i32, height: i32, indexed: bool) -> Self {
        Self::new_dims_indexed_buff(width, height, indexed, true)
    }

    pub fn default() -> Self {
        Self {
            bmp_buffer_data: Vec::new(),
            indexed_bmp_data: Vec::new(),
            width: 0,
            height: 0,
            stride: 0,
            indexed_stride: 0,
            bitmap_type: BitmapTypes::None,
            x_position: 0,
            y_position: 0,
            resolution: 0,
            texture: None,
        }
    }

    fn new_dims_indexed_buff(width: i32, height: i32, indexed: bool, bmp_buff: bool) -> Self {
        assert!(width >= 0 && height >= 0, "Negative bitmap8 dimensions");
        let stride = width;

        Self {
            bmp_buffer_data: if bmp_buff {
                vec![ColorRgba::black(); (height * stride) as usize]
            } else {
                vec![]
            },
            indexed_bmp_data: if indexed {
                vec![0; (stride * height) as usize]
            } else {
                vec![]
            },
            width,
            height,
            stride,
            indexed_stride: stride,
            bitmap_type: BitmapTypes::DibBitmap,
            x_position: 0,
            y_position: 0,
            resolution: 0,
            texture: None,
        }
    }

    pub fn create_texture(
        &mut self,
        scale_hint: &str,
        access: i32,
        renderer: &Option<SdlRendererPtr>,
    ) {
        if let Some(texture) = self.texture.as_mut() {
            unsafe { SDL_DestroyTexture(texture.0) };
        }
        let c_scale_hint = CString::new(scale_hint).expect("CString::new failed, invalid string");
        unsafe {
            SDL_SetHint(
                SDL_HINT_RENDER_SCALE_QUALITY.as_ptr() as *const i8,
                c_scale_hint.as_ptr(),
            );
        }

        if let Some(renderer) = renderer.as_ref() {
            unsafe {
                let tex = SDL_CreateTexture(
                    renderer.0,
                    PixelFormatEnum::BGRA32 as u32,
                    access,
                    self.width,
                    self.height,
                );
                self.texture = Some(SdlTexturePtr(tex));
                SDL_SetTextureBlendMode(tex, SDL_BLENDMODE_NONE);
            };
        }
    }
}

pub fn display_palette(plt: Option<&[ColorRgba]>, pb_game_state: &mut PbGameState) {
    const SYS_PALETTE_COLORS: [ColorRgba; 10] = [
        ColorRgba::color_rgba(0, 0, 0, 0),
        ColorRgba::color_rgba(0x80, 0, 0, 0xff),
        ColorRgba::color_rgba(0, 0x80, 0, 0xff),
        ColorRgba::color_rgba(0x80, 0x80, 0, 0xff),
        ColorRgba::color_rgba(0x0, 0, 0x80, 0xff),
        ColorRgba::color_rgba(0x80, 0, 0x80, 0xff),
        ColorRgba::color_rgba(0x0, 0x80, 0x80, 0xff),
        ColorRgba::color_rgba(0xC0, 0xC0, 0xC0, 0xff),
        ColorRgba::color_rgba(0xC0, 0xDC, 0xC0, 0xff),
        ColorRgba::color_rgba(0xA6, 0xCA, 0xF0, 0xff),
    ];

    pb_game_state.current_palette[..10].copy_from_slice(&SYS_PALETTE_COLORS);

    if let Some(plt_slice) = plt {
        for index in 10..246 {
            if index < plt_slice.len() {
                let mut src_clr = plt_slice[index];
                src_clr.set_alpha(0xff);

                let mut current_clr = src_clr;
                current_clr.set_alpha(2);

                pb_game_state.current_palette[index] = current_clr;
            }
        }
    }

    pb_game_state.current_palette[255] = ColorRgba::white();

    if let Some(table_arc) = pb_game_state.record_table.as_mut() {
        let mut t = table_arc.write().unwrap();
        for group in &mut t.groups {
            for i in 0..=2 {
                let bmp = group.get_bitmap_mut(i);
                apply_palette(bmp, &pb_game_state.current_palette);
            }
        }
    }
}

fn apply_palette(bmp: &mut GdrvBitmap8, palette: &[ColorRgba; 256]) {
    if bmp.bitmap_type == BitmapTypes::None {
        return;
    }
    if bmp.bitmap_type == BitmapTypes::Spliced {
        panic!("Wrong bitmap type");
    }

    let src_stride = bmp.indexed_stride as usize;
    let dst_stride = bmp.stride as usize;
    let height = bmp.height as usize;

    let dst_rows = bmp.bmp_buffer_data.chunks_exact_mut(dst_stride);
    let src_rows = bmp
        .indexed_bmp_data
        .chunks_exact(src_stride)
        .take(height)
        .rev();

    for (dst_row, src_row) in dst_rows.zip(src_rows) {
        for (dst_pixel, &src_pixel) in dst_row.iter_mut().zip(src_row.iter()) {
            *dst_pixel = palette[src_pixel as usize];
        }
    }
}

pub fn copy_bitmap(
    dst_bmp: &mut GdrvBitmap8,
    width: i32,
    height: i32,
    x_off: i32,
    y_off: i32,
    src_bmp: &GdrvBitmap8,
    src_x_off: i32,
    src_y_off: i32,
) {
    let mut src_index = (src_bmp.stride * src_y_off + src_x_off) as usize;
    let mut dst_index = (dst_bmp.stride * y_off + x_off) as usize;

    let width = width as usize;
    let src_stride = src_bmp.stride as usize;
    let dst_stride = dst_bmp.stride as usize;

    for _ in 0..height {
        let src_slice = &src_bmp.bmp_buffer_data[src_index..width + src_index];
        let dst_slice = &mut dst_bmp.bmp_buffer_data[dst_index..width + dst_index];

        dst_slice.copy_from_slice(src_slice);

        src_index += src_stride;
        dst_index += dst_stride;
    }
}

pub fn fill_bitmap(
    bmp: &mut GdrvBitmap8,
    width: i32,
    height: i32,
    x_off: i32,
    y_off: i32,
    fill_char: u8,
    pb_game_state: &mut PbGameState,
) {
    fill_bitmap_color_rgba(
        bmp,
        width,
        height,
        x_off,
        y_off,
        pb_game_state.current_palette[fill_char as usize],
    );
}

fn fill_bitmap_color_rgba(
    bmp: &mut GdrvBitmap8,
    width: i32,
    height: i32,
    x_off: i32,
    y_off: i32,
    fill_color: ColorRgba,
) {
    let mut index = bmp.stride * y_off + x_off;
    for _ in 0..height {
        for _x in (0..width).rev() {
            bmp.bmp_buffer_data[index as usize] = fill_color;
            index += 1;
        }
        index += bmp.stride - width;
    }
}

pub(crate) fn gr_text_draw_ttext_in_box(
    render_state: &mut RenderState,
    pb_game_state: &mut PbGameState,
    ui: &mut Ui,
) {
    let text_boxes = [
        pb_game_state.miss_text_box.as_mut().unwrap(),
        pb_game_state.info_text_box.as_mut().unwrap(),
    ];

    for text_box in text_boxes.into_iter() {
        unsafe { text_box.draw_im_gui(render_state, pb_game_state.text_box_color, ui) };
    }
}

pub(crate) fn copy_bitmap_w_transparency(
    dst_bmp: &mut GdrvBitmap8,
    width: i32,
    height: i32,
    x_off: i32,
    y_off: i32,
    src_bmp: &GdrvBitmap8,
    src_x_off: i32,
    src_y_off: i32,
) {
    let mut src_index = (src_bmp.stride * src_y_off + src_x_off) as usize;
    let mut dst_index = (dst_bmp.stride * y_off + x_off) as usize;

    let mut y = height;
    while y > 0 {
        let mut x = width;
        while x > 0 {
            if src_bmp.bmp_buffer_data[src_index].color > 0 {
                dst_bmp.bmp_buffer_data[dst_index] = src_bmp.bmp_buffer_data[src_index];
            }
            src_index += 1;
            dst_index += 1;
            x -= 1;
        }
        src_index += (src_bmp.stride - width) as usize;
        dst_index += (dst_bmp.stride - height) as usize;
        y -= 1;
    }
}
