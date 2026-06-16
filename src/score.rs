use crate::errors::ScoreError;
use crate::fullscrn;
use crate::gdrv::GdrvBitmap8;
use crate::group_data::{DatFile, EntryBuffer, FieldTypes};
use crate::pinball_state::FullscrnState;
use std::sync::{Arc, Mutex};

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

#[derive(Clone)]
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

pub static MSG_FONTP: Mutex<Option<ScoreMessageFontType>> = Mutex::new(None);

pub fn load_msg_font(
    font_name: &str,
    record_table: &mut Option<Arc<DatFile>>,
    fullscrn_state: &mut FullscrnState,
) -> Result<(), ScoreError> {
    let record_table = match &*record_table {
        Some(record_table) => record_table,
        None => return Ok(()),
    };

    let group_index = record_table.record_labeled(font_name);
    if group_index < 0 {
        return Ok(());
    }

    let mut msg_fontp = MSG_FONTP.lock().map_err(ScoreError::MsgFontLock)?;

    *msg_fontp = Some(ScoreMessageFontType::new());
    let font = msg_fontp.as_mut().unwrap();

    // FT font has multiple resolutions
    let gap_array = record_table.field(group_index, FieldTypes::ShortArray);
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
        let bmp = record_table.get_bitmap(group_index as i32, fullscrn_state.resolution);
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

pub(crate) fn unload_msg_font() {
    todo!()
}
