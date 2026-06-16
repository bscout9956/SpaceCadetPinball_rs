use crate::gdrv::GdrvBitmap8;
use crate::maths::RectangleType;
use crate::render::RenderSprite;
use crate::zdrv::ZMapHeaderType;
use sdl2::sys::SDL_Rect;
use std::sync::Arc;

#[derive(Default)]
pub struct RenderState {
    pub sprite_list: Vec<RenderSprite>,
    pub ball_list: Vec<RenderSprite>,
    pub v_screen: Option<GdrvBitmap8>,
    pub offset_x: i32,
    pub offset_y: i32,
    pub z_screen: Option<ZMapHeaderType>,
    pub z_map_offset_x: i32,
    pub z_map_offset_y: i32,
    pub background_bitmap: Option<GdrvBitmap8>,
    pub background_zmap: Option<Arc<ZMapHeaderType>>,
    pub destination_rect: Option<SDL_Rect>,
    pub v_screen_rect: RectangleType,
    pub ball_bitmap: Option<[GdrvBitmap8; 20]>,
}
