use crate::errors::TTextBoxError;
use crate::gdrv::GdrvBitmap8;
use crate::score::ScoreMessageFontType;
use crate::state::loader_state::LoaderState;
use crate::state::pb_game_state::PbGameState;
use crate::state::render_state::RenderState;
use crate::state::score_state::ScoreState;
use crate::t_pinball_table::TPinballTable;
use crate::t_textbox_message::TTextBoxMessage;
use crate::{fullscrn, loader, pb};
use dear_imgui_rs::sys::{
    ImGuiCol_Text, ImGuiViewportFlags_NoFocusOnAppearing, ImGuiWindowFlags_NoBackground,
    ImGuiWindowFlags_NoDecoration, ImGuiWindowFlags_NoFocusOnAppearing, ImGuiWindowFlags_NoInputs,
    ImGuiWindowFlags_NoSavedSettings, ImU32, ImVec2, igBegin, igEnd, igPopStyleColor,
    igPushStyleColor_U32, igSetNextWindowPos, igSetNextWindowSize, igTextWrapped,
};
use dear_imgui_rs::{Context, Ui};
use sdl2::sys::SDL_Rect;
use std::cell::RefCell;
use std::ffi::CString;
use std::ptr::null_mut;
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
    fn draw(&mut self, state: &mut PinballState) {
        if let Some(v_screen) = state.render_state.v_screen.as_mut() {
            if let Some(bg) = self.bg_bmp.as_mut() {
                gdrv::copy_bitmap(
                    v_screen,
                    self.width,
                    self.height,
                    self.offset_x,
                    self.offset_y,
                    bg,
                    self.offset_x,
                    self.offset_y,
                )
            } else {
                gdrv::fill_bitmap(
                    v_screen,
                    self.width,
                    self.height,
                    self.offset_x,
                    self.offset_y,
                    0,
                    &mut state.pb_game_state,
                )
            }

            let mut display = false;
            while let Some(front_msg) = self.messages.front() {
                if front_msg.time == -1.0f32 {
                    if self.messages.len() <= 1 {
                        self.timer = -1;
                        display = true;
                        break;
                    }
                } else if front_msg.time_left(state.pb_game_state.time_ticks) >= -2.0f32 {
                    self.timer = timer::set(
                        f32::max(front_msg.time_left(state.pb_game_state.time_ticks), 0.25f32),
                        &raw const *self as *mut c_void,
                        Self::timer_expired,
                        state.pb_game_state.time_ticks,
                    );
                    display = true;
                    break;
                }

                self.messages.pop_front();
            }

            if display {
                let font = match self.font.as_ref() {
                    None => return,
                    Some(f) => f,
                };

                if let Some(front_msg) = self.messages.front() {
                    let mut lines = Vec::new();
                    let mut text_height = 0;
                    let mut remaining_text = front_msg.text;

                    while !remaining_text.is_empty() {
                        if text_height + font.height > self.height {
                            break;
                        }

                        let result = self.layout_text_line(remaining_text);
                        let result_end = result.end;

                        if result.start.is_empty() && result.end == remaining_text {
                            break;
                        }

                        lines.push(result);
                        remaining_text = result_end;
                        text_height += font.height;
                    }

                    let mut off_y = self.offset_y;
                    if state.pb_game_state.full_tilt_mode {
                        off_y += (self.height - text_height) / 2;
                    }
                    for line in lines {
                        let mut off_x = self.offset_x;
                        if state.pb_game_state.full_tilt_mode {
                            off_x += (self.width - line.width) / 2;
                        }
                        for &char_byte in line.start.as_bytes() {
                            let masked_char = (char_byte & 0x7f) as usize;
                            let char_bmp = &font.chars[masked_char];

                            if char_bmp.height > 0 {
                                let height = char_bmp.height;
                                let width = char_bmp.width;
                                if let Some(_bg_bmp) = state.render_state.background_bitmap.as_ref()
                                {
                                    gdrv::copy_bitmap_w_transparency(
                                        v_screen, width, height, off_x, off_y, char_bmp, 0, 0,
                                    );
                                } else {
                                    gdrv::copy_bitmap(
                                        v_screen, width, height, off_x, off_y, char_bmp, 0, 0,
                                    );
                                }
                                off_y += char_bmp.width + font.gap_width;
                            }
                        }
                        off_y += font.height;
                    }
                }
            }
        }
    }
}

impl TTextBox {
    pub(crate) unsafe fn draw_im_gui(
        &self,
        render_state: &mut RenderState,
        text_box_color: ImU32,
        ui: &mut Ui,
    ) {
        if self.font.is_some() || !self.current_message.is_some() {
            return;
        }

        let window_flags = ImGuiWindowFlags_NoBackground
            | ImGuiWindowFlags_NoDecoration
            | ImGuiWindowFlags_NoSavedSettings
            | ImGuiWindowFlags_NoFocusOnAppearing
            | ImGuiWindowFlags_NoInputs;

        let mut rect = SDL_Rect {
            x: self.offset_x,
            y: self.offset_y,
            w: self.width,
            h: self.height,
        };

        rect = fullscrn::get_screen_rect_from_pinball_rect(rect, render_state);
        unsafe {
            igSetNextWindowPos(
                ImVec2::new(rect.x as f32, rect.y as f32),
                0,
                ImVec2::new(0.0, 0.0),
            );
            igSetNextWindowSize(ImVec2::new(rect.w as f32, rect.h as f32), 0);
            let window_name = format!("TTextbox_{:p}", self);
            let win_cstr = CString::new(window_name).unwrap();
            if igBegin(win_cstr.as_ptr(), null_mut(), window_flags) {
                ui.set_window_font_scale(fullscrn::get_screen_to_pinball_ratio());

                igPushStyleColor_U32(ImGuiCol_Text, text_box_color);
                igTextWrapped(
                    c"%s".as_ptr(),
                    self.current_message.as_ref().unwrap().text.as_ptr(),
                );
                // TODO: Check if you need to remove this, might have...
                igPopStyleColor(1);
            }
            igEnd();
        }
    }
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
