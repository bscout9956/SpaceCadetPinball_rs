use dear_imgui_rs::Io;
use dear_imgui_rs::{Context, TextureId};
use sdl2::render::RenderTarget;
use sdl2::sys::{
    SDL_CreateRGBSurface, SDL_CreateRGBSurfaceFrom, SDL_CreateTexture,
    SDL_CreateTextureFromSurface, SDL_DestroyTexture, SDL_FreeSurface, SDL_Renderer, SDL_Surface,
    SDL_Texture,
};
use std::ffi::c_void;
use std::ops::{Add, Mul};
use std::sync::{LazyLock, LockResult, Mutex};

pub static CURRENT_DEVICE: LazyLock<Mutex<Option<Device>>> = LazyLock::new(|| Mutex::new(None));

struct ClipRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

struct TriangleCacheItem {
    texture: *mut SDL_Texture,
    width: i32,
    height: i32,
}

impl Drop for TriangleCacheItem {
    fn drop(&mut self) {
        if !self.texture.is_null() {
            unsafe {
                SDL_DestroyTexture(self.texture);
            }
        }
    }
}

type Clip = ClipRect;

static UNIFORM_COLOR_TRIANGLE_CACHE_SIZE: usize = 512;
static GENERIC_TRIANGLE_CACHE_SIZE: usize = 64;

pub struct Device {
    pub renderer: *mut SDL_Renderer,
    pub cache_was_invalidated: bool,
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

type UniformColorTriangleKey = (u32, i32, i32, i32, i32, i32, i32);
type GenericTriangleVertexKey = (i32, i32, f64, f64, u32);
type GenericTriangleKey = (
    GenericTriangleVertexKey,
    GenericTriangleVertexKey,
    GenericTriangleVertexKey,
);
impl Device {
    pub fn new(renderer: *mut SDL_Renderer) -> Self {
        Self {
            renderer,
            cache_was_invalidated: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn from_u32(color: u32) -> Self {
        Self {
            r: ((color >> 0) & 0xff) as f32 / 255.0,
            g: ((color >> 8) & 0xff) as f32 / 255.0,
            b: ((color >> 16) & 0xff) as f32 / 255.0,
            a: ((color >> 24) & 0xff) as f32 / 255.0,
        }
    }

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_int(&self) -> u32 {
        (((self.r * 255.0) as u32 & 0xff) << 0)
            | (((self.g * 255.0) as u32 & 0xff) << 8)
            | (((self.b * 255.0) as u32 & 0xff) << 16)
            | (((self.a * 255.0) as u32 & 0xff) << 24)
    }

    pub fn use_as_draw_color<T: RenderTarget>(&self, canvas: &mut sdl2::render::Canvas<T>) {
        canvas.set_draw_color(sdl2::pixels::Color::RGBA(
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ));
    }
}

impl Mul<Color> for Color {
    type Output = Self;

    fn mul(self, rhs: Color) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
            a: self.a * rhs.a,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

impl Add<Color> for Color {
    type Output = Self;

    fn add(self, rhs: Color) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

struct Texture {
    surface: *mut SDL_Surface,
    source: *mut SDL_Texture,
}

impl Texture {
    pub unsafe fn sample(&self, u: f32, v: f32) -> Color {
        unsafe {
            let x: i32 = f32::round(u * ((*self.surface).w - 1) as f32 + 0.5) as i32;
            let y: i32 = f32::round(v * ((*self.surface).h - 1) as f32 + 0.5) as i32;

            let location = y * (*self.surface).w + x;
            assert!(location < (*self.surface).w * (*self.surface).h);
            let color_u32 = unsafe {
                let ptr = (*self.surface).pixels as *const u32;
                ptr.offset(location as isize).read()
            };

            Color::from_u32(color_u32)
        }
    }

    pub fn new(surface: *mut SDL_Surface, source: *mut SDL_Texture) -> Self {
        Self { surface, source }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            SDL_FreeSurface(self.surface);
            SDL_DestroyTexture(self.source);
        }
    }
}

pub fn initialize(
    context: &mut Context,
    renderer: *mut SDL_Renderer,
    window_width: i32,
    window_height: i32,
) {
    let mut io = context.io_mut();
    io.set_display_size([window_width as f32, window_height as f32]);
    let context_style = context.style_mut();
    context_style.set_window_rounding(0.0);
    context_style.set_anti_aliased_fill(false);
    context_style.set_anti_aliased_lines(false);
    context_style.set_child_rounding(0.0);
    context_style.set_popup_rounding(0.0);
    context_style.set_frame_rounding(0.0);
    context_style.set_scrollbar_rounding(0.0);
    context_style.set_grab_rounding(0.0);
    context_style.set_tab_rounding(0.0);

    let tex_data = context.fonts().get_tex_data();
    unsafe {
        let (pixels, width, height) = ((*tex_data).Pixels, (*tex_data).Width, (*tex_data).Height);
        let rmask = 0x000000ff;
        let gmask = 0x0000ff00;
        let bmask = 0x00ff0000;
        let amask = 0xff000000;

        let surface = SDL_CreateRGBSurfaceFrom(
            pixels as *mut c_void,
            width,
            height,
            32,
            4 * width,
            rmask,
            gmask,
            bmask,
            amask,
        );

        let source = SDL_CreateTextureFromSurface(renderer, surface);
        let texture = Box::new(Texture::new(surface, source));
        let raw_addr = Box::into_raw(texture) as usize;
        let texture_id = TextureId::new(raw_addr as u64);
        context.fonts().set_texture_id(texture_id);
        if let Ok(mut guard) = CURRENT_DEVICE.lock() {
            *guard = Some(Device::new(renderer));
        }
    }
}
