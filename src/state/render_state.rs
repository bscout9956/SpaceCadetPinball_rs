use crate::gdrv::GdrvBitmap8;
use crate::render::RenderSprite;
use crate::zdrv::ZMapHeaderType;

pub struct RenderState {
    pub sprite_list: Vec<RenderSprite>,
    pub ball_list: Vec<RenderSprite>,
    pub v_screen: Option<GdrvBitmap8>,
    pub offset_x: i32,
    pub offset_y: i32,
    pub z_screen: Option<ZMapHeaderType>,
}

impl RenderState {
    pub(crate) fn new() -> Self {
        Self {
            sprite_list: Vec::new(),
            ball_list: Vec::new(),
            v_screen: None,
            offset_x: 0,
            offset_y: 0,
            z_screen: None,
        }
    }
}
