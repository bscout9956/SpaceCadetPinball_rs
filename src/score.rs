use crate::errors::ScoreError;
use crate::gdrv::GdrvBitmap8;
use crate::group_data::{DatFile, EntryBuffer, FieldTypes};
use crate::state::fullscrn_state::FullscrnState;
use crate::state::pinball_state::PinballState;
use crate::state::score_state::ScoreState;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
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
    pub chars: Vec<GdrvBitmap8>,
}

impl ScoreMessageFontType {
    fn new() -> Self {
        Self {
            gap_width: 0,
            height: 0,
            chars: Vec::with_capacity(128),
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
    let mut score = ScoreStruct::default();
    score.score = -9999;
    score.background_bmp = render_bg_bmp?;

    /*Full tilt: score box dimensions index is offset by resolution*/
    // TODO: moved here issue
    if let Some(rt_arc) = state.pb_game_state.record_table.as_ref()
        && let Some(lt_arc) = state.loader_state.loader_table.as_ref()
    {
        let rt = rt_arc.read().unwrap();
        let lt = lt_arc.read().unwrap();
        let dimensions_id = rt.record_labeled(field_name);
        let dimensions = lt.field(dimensions_id, FieldTypes::ShortArray);
        if let Some(raw_data) = dimensions {
            match raw_data {
                EntryBuffer::Raw(vec_data) => {
                    let mut group_index = vec_data[0] as i32;
                    score.offset_x =
                        i32::from_le_bytes([vec_data[0], vec_data[1], vec_data[2], vec_data[3]]);
                    score.offset_y =
                        i32::from_le_bytes([vec_data[4], vec_data[5], vec_data[6], vec_data[7]]);
                    score.width =
                        i32::from_le_bytes([vec_data[8], vec_data[9], vec_data[10], vec_data[11]]);
                    score.height = i32::from_le_bytes([
                        vec_data[12],
                        vec_data[13],
                        vec_data[14],
                        vec_data[15],
                    ]);

                    for index in 0..10 {
                        score.char_bmp[index] = lt
                            .get_bitmap(group_index, state.fullscrn_state.resolution)
                            .clone();
                        group_index += 1;
                    }
                }
                _ => {
                    return None;
                }
            }
        } else {
            //TODO: score drop?
            return None;
        }
    }
    Some(score)
}

pub(crate) fn dup(score_struct: Option<ScoreStruct>, p1: usize) -> ScoreStruct {
    ScoreStruct::from(score_struct.unwrap())
}
