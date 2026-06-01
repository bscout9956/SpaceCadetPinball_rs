//todo: temporary

use crate::gdrv::GdrvBitmap8;

pub struct ScoreStruct {
    pub score: i32,
    pub dirty_flag: bool,
    pub background_bmp: GdrvBitmap8,
    pub offset_x: i32,
    pub offset_y: i32,
    pub width: i32,
    pub height: i32,
    pub char_bmp: [GdrvBitmap8; 10],
    pub msg_font: &'static ScoreMessageFontType,
}

pub struct ScoreMessageFontType {
    pub gap_width: i32,
    pub height: i32,
    pub chars: [GdrvBitmap8; 128],
}

// TODO: Implement everything else
