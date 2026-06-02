use bytemuck::{Pod, Zeroable};
use sdl2::sys::SDL_Texture;

#[derive(Clone)]
pub struct ZMapHeaderType {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub resolution: u32,
    pub z_map_data: Vec<u16>,
    pub texture: Option<SDL_Texture>,
}

impl ZMapHeaderType {
    pub fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            stride: 0,
            resolution: 0,
            z_map_data: vec![],
            texture: None,
        }
    }

    pub fn new(width: i32, height: i32, stride: i32) -> Self {
        Self {
            width,
            height,
            stride,
            resolution: 0,
            z_map_data: vec![],
            texture: None,
        }
    }
}
