use crate::errors::ScoreError;
use crate::gdrv::GdrvBitmap8;
use crate::group_data::{DatFile, EntryBuffer, FieldTypes};
use crate::state::fullscrn_state::FullscrnState;
use crate::state::score_state::ScoreState;
use std::sync::{Arc, RwLock};

pub struct ScoreStruct {
    pub score: i32,
    pub dirty_flag: bool,
    pub background_bmp: GdrvBitmap8,
    pub offset_x: i32,
    pub offset_y: i32,
    pub width: i32,
    pub height: i32,
    pub char_bmp: [GdrvBitmap8; 10],
    pub msg_font: ScoreMessageFontType,
}

impl Default for ScoreStruct {
    fn default() -> ScoreStruct {
        Self {
            score: 0,
            dirty_flag: false,
            background_bmp: GdrvBitmap8::default(),
            offset_x: 0,
            offset_y: 0,
            width: 0,
            height: 0,
            char_bmp: std::array::from_fn(|_| GdrvBitmap8::default()),
            msg_font: ScoreMessageFontType::new(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ScoreMessageFontType {
    pub gap_width: i32,
    pub height: i32,
    pub chars: [GdrvBitmap8; 128],
}

impl ScoreMessageFontType {
    fn new() -> Self {
        Self {
            gap_width: 0,
            height: 0,
            chars: std::array::from_fn(|_| GdrvBitmap8::default()),
        }
    }
}

pub fn load_msg_font(
    font_name: &str,
    record_table: &mut Option<Arc<RwLock<DatFile>>>,
    fullscrn_state: &mut FullscrnState,
    score_state: &mut ScoreState,
) -> Result<(), ScoreError> {
    let t = record_table.as_ref().unwrap().write().unwrap();

    let group_index = t.record_labeled(font_name);
    if group_index < 0 {
        return Ok(());
    }

    let font = score_state.MSG_FONTP.as_mut().unwrap();

    // FT font has multiple resolutions
    let gap_array = t.field(group_index, FieldTypes::ShortArray);
    if let Some(EntryBuffer::Raw(bytes)) = gap_array {
        let res_val = fullscrn_state.resolution;
        let offset = res_val as usize * 2;

        if offset + 1 < bytes.len() {
            font.gap_width = i16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as i32;
        }
    } else {
        font.gap_width = 0;
    }

    for (char_index, group_index) in (32..128).zip(group_index as usize..) {
        let bmp = t.get_bitmap(group_index as i32, fullscrn_state.resolution);
        // TODO: This is an assumption, but it may work? Get bitmap doesn't return an option so idk
        if bmp.indexed_bmp_data.is_empty() {
            break;
        }
        if font.height == 0 {
            font.height = bmp.height;
        }
        font.chars[char_index] = bmp.clone();
    }

    Ok(())
}

pub fn init() -> i32 {
    1
}
