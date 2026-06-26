use crate::gdrv::GdrvBitmap8;

pub struct DebugState {
    pub db_screen: Option<GdrvBitmap8>,
}

impl DebugState {
    pub fn new() -> DebugState {
        Self { db_screen: None }
    }
}
