use crate::errors::ScoreError;
use crate::gdrv;
use crate::gdrv::{ColorRgba, GdrvBitmap8};
use crate::group_data::{DatFile, EntryBuffer, FieldTypes};
use crate::state::fullscrn_state::FullscrnState;
use crate::state::pinball_state::PinballState;
use crate::state::score_state::ScoreState;
use anyhow::{Result, bail};
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
    pub char_bmp: Vec<Option<GdrvBitmap8>>,
    pub msg_font: ScoreMessageFontType,
}

impl Default for ScoreStruct {
    fn default() -> ScoreStruct {
        Self {
            score: 0,
            dirty_flag: false,
            background_bmp: None,
            offset_x: 0,
            offset_y: 0,
            width: 0,
            height: 0,
            char_bmp: Vec::new(),
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

    score_state.MSG_FONTP = Some(ScoreMessageFontType::new());
    let font = score_state.MSG_FONTP.as_mut().unwrap();
    font.chars = vec![GdrvBitmap8::default(); 128];

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
        if let Some(b) = bmp {
            if font.height == 0 {
                font.height = b.height;
            }
            font.chars[char_index] = b.clone();
        }
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
        char_bmp: vec![None; 11],
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
    // TODO: p1? Should it be used?
    score_struct.unwrap()
}

pub(crate) fn set(score: &mut ScoreStruct, value: i32) {
    score.score = value;
    score.dirty_flag = true;
}

pub(crate) fn update(
    score_opt: &mut Option<ScoreStruct>,
    v_screen: &mut Option<GdrvBitmap8>,
    current_palette: &[ColorRgba; 256],
    background_bitmap: &Option<GdrvBitmap8>,
) -> Result<()> {
    let score_str;
    if let Some(score) = score_opt.as_mut()
        && score.dirty_flag
        && score.score <= 1000000000
    {
        score.dirty_flag = false;
        let mut x = score.width + score.offset_x;
        let y = score.offset_y;
        erase(score, 0, v_screen, current_palette)?;
        if score.score >= 0 {
            score_str = format!("{}", score.score);
            for &cur_char in score_str.as_bytes().iter().rev() {
                let digit = cur_char - b'0';
                let bmp = score.char_bmp[(digit % 10) as usize].as_ref();
                if let Some(b) = bmp {
                    x -= b.width;
                    let height = b.height;
                    let width = b.width;
                    if let Some(vscr) = v_screen.as_mut() {
                        // I guess we just add transparency but discard the bg bmp???
                        if let Some(_bg_bmp) = background_bitmap.as_ref() {
                            gdrv::copy_bitmap_w_transparency(vscr, width, height, x, y, b, 0, 0);
                        } else {
                            gdrv::copy_bitmap(vscr, width, height, x, y, b, 0, 0)?;
                        }
                    }
                } else {
                    bail!("No bitmap to subtract x from");
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn erase(
    score: &mut ScoreStruct,
    _blit_flag: i32,
    v_screen: &mut Option<GdrvBitmap8>,
    current_palette: &[ColorRgba; 256],
) -> Result<()> {
    if let Some(vscr) = v_screen.as_mut() {
        if let Some(bg_bmp) = score.background_bmp.as_mut() {
            gdrv::copy_bitmap(
                vscr,
                score.width,
                score.height,
                score.offset_x,
                score.offset_y,
                bg_bmp,
                score.offset_x,
                score.offset_y,
            )?;
        } else {
            gdrv::fill_bitmap(
                vscr,
                score.width,
                score.height,
                score.offset_x,
                score.offset_y,
                0,
                current_palette,
            )?;
        }
    }
    Ok(())
}
