//todo: temporary
struct GdrvBitmap8;

pub struct ScoreStruct {
    score: i32,
    dirty_flag: bool,
    background_bmp: GdrvBitmap8,
    offset_x:i32,
    offset_y:i32,
    width:i32,
    height:i32,
    char_bmp: [GdrvBitmap8; 10],
    msg_font: &'static ScoreMessageFontType,
}

pub struct ScoreMessageFontType {
    pub gap_width: i32,
    pub height: i32,
    pub chars: [GdrvBitmap8; 128]
}

// TODO: Implement everything else