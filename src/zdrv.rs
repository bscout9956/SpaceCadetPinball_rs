use sdl2::sys::SDL_Texture;

pub struct ZMapHeaderType {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub resolution: u32,
    pub z_ptr_1: u16,
    pub texture: SDL_Texture,
}
