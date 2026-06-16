use crate::errors::TTextBoxError;
use crate::gdrv::GdrvBitmap8;
use crate::score::ScoreMessageFontType;
use crate::state::loader_state::LoaderState;
use crate::state::score_state::ScoreState;
use crate::t_pinball_table::TPinballTable;
use crate::t_textbox_message::TTextBoxMessage;
use crate::loader;
use std::cell::RefCell;
use std::rc::Weak;
use std::slice;
pub struct TTextBox {
    pub offset_x: i32,
    pub offset_y: i32,
    pub width: i32,
    pub height: i32,
    pub timer: i32,
    pub bg_bmp: Option<GdrvBitmap8>,
    pub font: Option<ScoreMessageFontType>,
    pub current_message: Option<TTextBoxMessage>,
    pub previous_message: Option<TTextBoxMessage>,
}

impl TTextBox {
    pub(crate) fn clear(&self, p0: bool) {
        todo!()
    }
}

impl TTextBox {
    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        resolution: i32,
        background_bitmap: Option<GdrvBitmap8>,
        loader_state: &mut LoaderState,
        score_state: &mut ScoreState,
    ) -> Result<TTextBox, TTextBoxError> {
        let mut instance = Self {
            offset_x: 0,
            offset_y: 0,
            width: 0,
            height: 0,
            timer: 0,
            bg_bmp: background_bitmap.clone(),
            font: score_state.MSG_FONTP.clone(),
            current_message: None,
            previous_message: None,
        };

        if group_index > 0 {
            /*Full tilt: text box dimensions index is offset by resolution*/
            let mut arr_length: i32 = 0;
            let dimensions = loader::query_int_attribute(
                group_index + resolution,
                1500,
                &mut arr_length,
                loader_state,
            )?;

            let dim_array =
                unsafe { slice::from_raw_parts(dimensions as *mut i16, arr_length as usize) };

            if dim_array.len() < 4 {
                panic!("dim array length is too small");
            }

            instance.offset_x = dim_array[0] as i32;
            instance.offset_y = dim_array[1] as i32;
            instance.width = dim_array[2] as i32;
            instance.height = dim_array[3] as i32;
        }

        Ok(instance)
    }
}

struct LayoutResult {
    start: &'static str,
    end: &'static str,
    width: i32,
}
