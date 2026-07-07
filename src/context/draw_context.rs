use crate::gdrv::{ColorRgba, GdrvBitmap8};

pub struct DrawContext<'a> {
    pub v_screen: &'a mut Option<GdrvBitmap8>,
    pub current_palette: &'a [ColorRgba; 256],
    pub background_bitmap: &'a Option<GdrvBitmap8>,
}