use sdl2::sys::SDL_Texture;

#[derive(Copy, Clone)]
pub struct ZMapHeaderType {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub resolution: u32,
    pub z_ptr_1: u16,
    pub texture: Option<SDL_Texture>,
}

impl ZMapHeaderType {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            stride: 0,
            resolution: 0,
            z_ptr_1: 0,
            texture: None,
        }
    }
}
