//todo: temporary

use crate::gdrv::GdrvBitmap8;
use crate::group_data::{DatFile, EntryBuffer, FieldTypes};
use crate::pb::RECORD_TABLE;
use crate::{fullscrn, pb};
use dear_imgui_rs::table;
use std::sync::{LazyLock, LockResult, Mutex, MutexGuard, PoisonError, TryLockResult};
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum ScoreError {
    #[error("Failed to lock RecordTable from PB: `{0}`")]
    RecordTableLock(#[from] PoisonError<MutexGuard<'static, Option<DatFile>>>),
    #[error("Failed to lock MSG_FONTP from Score: `{0}`")]
    MsgFontLock(#[from] PoisonError<MutexGuard<'static, Option<ScoreMessageFontType>>>),
}

pub fn load_msg_font(font_name: &str) -> Result<(), ScoreError> {
    let record_table = RECORD_TABLE.lock().map_err(ScoreError::RecordTableLock)?;

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
        let resolution = fullscrn::get_resolution() as usize;
        let offset = resolution * 2;

        if offset + 1 < bytes.len() {
            font.gap_width = i16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as i32;
        }
    } else {
        font.gap_width = 0;
    }

    for (char_index, group_index) in (32..128).zip(group_index as usize..) {
        let bmp = record_table.get_bitmap(group_index as i32);
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
