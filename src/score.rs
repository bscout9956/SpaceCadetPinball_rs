use crate::errors::ScoreError;
use crate::gdrv::GdrvBitmap8;
use crate::group_data::{DatFile, EntryBuffer, FieldTypes};
use crate::state::fullscrn_state::FullscrnState;
use crate::state::pinball_state::PinballState;
use crate::state::score_state::ScoreState;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct ScoreStruct {
    pub score: i32,
    pub dirty_flag: bool,
    pub background_bmp: Option<GdrvBitmap8>,
    pub offset_x: i32,
    pub offset_y: i32,
    pub width: i32,
    pub height: i32,
    pub char_bmp: Vec<GdrvBitmap8>,
    pub msg_font: ScoreMessageFontType,
}

impl Default for ScoreStruct {
    fn default() -> ScoreStruct {
        Self {
            score: 0,
            dirty_flag: false,
            background_bmp: Some(GdrvBitmap8::default()),
            offset_x: 0,
            offset_y: 0,
            width: 0,
            height: 0,
            char_bmp: (0..10).map(|_| GdrvBitmap8::default()).collect(),
            msg_font: ScoreMessageFontType::new(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ScoreMessageFontType {
    pub gap_width: i32,
    pub height: i32,
    pub chars: Vec<GdrvBitmap8>,
}

impl ScoreMessageFontType {
    fn new() -> Self {
        println!("Initializing the score message font");
        Self {
            gap_width: 0,
            height: 0,
            chars: Vec::new(),
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

pub(crate) fn create(
    field_name: &str,
    render_bg_bmp: Option<GdrvBitmap8>,
    state: &mut PinballState,
) -> Option<ScoreStruct> {
    let mut score = ScoreStruct {
        score: -9999,
        background_bmp: render_bg_bmp,
        ..Default::default()
    };

    /*Full tilt: score box dimensions index is offset by resolution*/
    if let Some(rt_arc) = state.pb_game_state.record_table.as_ref()
        && let Some(lt_arc) = state.loader_state.loader_table.as_ref()
    {
        let rt = rt_arc.read().unwrap();
        let lt = lt_arc.read().unwrap();
        let dimensions_id = rt.record_labeled(field_name) + state.fullscrn_state.resolution;
        let dimensions = lt.field(dimensions_id, FieldTypes::ShortArray);
        if let Some(raw_data) = dimensions {
            match raw_data {
                EntryBuffer::Raw(vec_data) => {
                    let mut group_index = vec_data[0] as i32;
                    let read_i16 = |start: usize| -> i16 {
                        i16::from_le_bytes(vec_data[start..start + 2].try_into().unwrap())
                    };

                    score.offset_x = read_i16(2) as i32;
                    score.offset_y = read_i16(4) as i32;
                    score.width = read_i16(6) as i32;
                    score.height = read_i16(8) as i32;

                    for index in 0..10 {
                        score.char_bmp[index] = lt
                            .get_bitmap(group_index, state.fullscrn_state.resolution)
                            .clone();
                        group_index += 1;
                    }
                }
                _ => {
                    eprintln!("Invalid state, no score");
                    return None;
                }
            }
        } else {
            //TODO: score drop?
            eprintln!("Invalid state, no score");
            return None;
        }
    } else {
        eprintln!("Invalid state, no loader/record table?");
    }
    Some(score)
}

pub(crate) fn dup(score_struct: Option<ScoreStruct>, p1: usize) -> ScoreStruct {
    ScoreStruct::from(score_struct.unwrap())
}

pub(crate) fn set(score: &mut Option<ScoreStruct>, value: i32) {
    if let Some(s) = score.as_mut() {
        s.score = value;
        s.dirty_flag = true;
    }
}
