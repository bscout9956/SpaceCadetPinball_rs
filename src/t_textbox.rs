use crate::gdrv::{ColorRgba, GdrvBitmap8};
use crate::score::ScoreMessageFontType;
use crate::state::loader_state::LoaderState;
use crate::state::pinball_state::PinballState;
use crate::state::render_state::RenderState;
use crate::state::score_state::ScoreState;
use crate::t_pinball_table::TPinballTable;
use crate::t_textbox_message::TTextBoxMessage;
use crate::utils::DrawContext;
use crate::{fullscrn, gdrv, loader, timer};
use anyhow::Context;
use anyhow::Result;
use dear_imgui_rs::Ui;
use dear_imgui_rs::sys::{
    ImGuiCol_Text, ImGuiWindowFlags_NoBackground, ImGuiWindowFlags_NoDecoration,
    ImGuiWindowFlags_NoFocusOnAppearing, ImGuiWindowFlags_NoInputs,
    ImGuiWindowFlags_NoSavedSettings, ImU32, ImVec2, igBegin, igEnd, igPopStyleColor,
    igPushStyleColor_U32, igSetNextWindowPos, igSetNextWindowSize, igTextWrapped,
};
use sdl2::sys::SDL_Rect;
use std::cell::RefCell;
use std::cmp::PartialEq;
use std::collections::VecDeque;
use std::ffi::{CString, c_void};
use std::ptr::null_mut;
use std::rc::Weak;
use std::slice;

#[derive(PartialEq, Clone)]
pub struct TTextBox {
    pub offset_x: i32,
    pub offset_y: i32,
    pub width: i32,
    pub height: i32,
    pub timer: i32,
    pub bg_bmp: Option<GdrvBitmap8>,
    pub font: Option<ScoreMessageFontType>,
    pub messages: VecDeque<TTextBoxMessage>,
}

impl TTextBox {
    pub(crate) fn display(
        &mut self,
        text: &str,
        time: f32,
        draw_context: &mut DrawContext,
        low_priority: Option<bool>,
    ) -> Result<()> {
        let prio = low_priority.unwrap_or(false);
        if text.is_empty() {
            return Ok(());
        }

        let is_dupe = self.messages.back().map_or(false, |msg| msg.text == text);

        if is_dupe {
            if let Some(prev_msg) = self.messages.back_mut() {
                prev_msg.refresh(time, draw_context.time_ticks);
            }

            if self.messages.len() == 1 {
                if self.timer > 0 && self.timer != -1 {
                    timer::kill_id(self.timer);
                }
                if time == -1.0f32 {
                    self.timer = -1;
                } else {
                    self.timer = timer::set(
                        time,
                        &raw const *self as *mut c_void,
                        Self::timer_expired,
                        draw_context,
                    );
                }
            }
        } else {
            if self.timer == -1 {
                self.clear(false);
            }
            let new_message = TTextBoxMessage::new(text, time, prio, draw_context.time_ticks);
            self.messages.push_back(new_message);

            if self.timer == 0 {
                self.draw(draw_context).context("Failed to draw TTextBox")?;
            }
        }
        Ok(())
    }

    pub unsafe extern "C" fn timer_expired(
        timer_id: i32,
        caller: *mut c_void,
        draw_ctx: &mut DrawContext,
    ) {
        let tb = unsafe { &mut *(caller as *mut TTextBox) };
        (*tb).timer = 0;
        if tb.messages.pop_front().is_some() {
            let _ = tb.draw(draw_ctx).context("Failed to draw textbox");
            // TODO: control shit
        }
    }

    fn draw(&mut self, draw_ctx: &mut DrawContext) -> Result<()> {
        if let Some(v_screen) = draw_ctx.v_screen.as_mut() {
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
                    draw_ctx.current_palette,
                )
                .context("Failed to fill bitmap for TTextBox")?;
            }

        let mut display = false;
        while let Some(front_msg) = self.messages.front() {
            if front_msg.time == -1.0f32 {
                if self.messages.len() <= 1 {
                    self.timer = -1;
                    display = true;
                    break;
                }
            } else if front_msg.time_left(draw_ctx.time_ticks) >= -2.0f32 {
                self.timer = timer::set(
                    f32::max(front_msg.time_left(draw_ctx.time_ticks), 0.25f32),
                    &raw const *self as *mut c_void,
                    Self::timer_expired,
                    draw_ctx,
                );
                display = true;
                break;
            }

            self.messages.pop_front();
        }

                if let Some(front_msg) = self.messages.front() {
                    let mut lines = Vec::new();
                    let mut text_height = 0;
                    let mut remaining_text = front_msg.text.as_str();

                    while !remaining_text.is_empty() {
                        if text_height + font.height > self.height {
                            break;
                        }

                        let result = self.layout_text_line(&remaining_text);
                        let result_end = result.end;

                        if result.start.is_empty() && result.end == remaining_text {
                            break;
                        }

                        lines.push(result);
                        remaining_text = result_end;
                        text_height += font.height;
                    }

                    let mut off_y = self.offset_y;
                    if full_tilt_mode {
                        off_y += (self.height - text_height) / 2;
                    }
                    for line in lines {
                        let mut off_x = self.offset_x;
                        if full_tilt_mode {
                            off_x += (self.width - line.width) / 2;
                        }
                        for &char_byte in line.start.as_bytes() {
                            let masked_char = (char_byte & 0x7f) as usize;
                            let char_bmp = &font.chars[masked_char];

                    if result.start.is_empty() && result.end == remaining_text {
                        break;
                    }

                    lines.push(result);
                    remaining_text = result_end;
                    text_height += font.height;
                }

                let mut off_y = self.offset_y;
                if draw_ctx.full_tilt_mode {
                    off_y += (self.height - text_height) / 2;
                }
                for line in lines {
                    let mut off_x = self.offset_x;
                    if draw_ctx.full_tilt_mode {
                        off_x += (self.width - line.width) / 2;
                    }
                    for &char_byte in line.start.as_bytes() {
                        let masked_char = (char_byte & 0x7f) as usize;
                        let char_bmp = &font.chars[masked_char];

                        if char_bmp.height > 0 {
                            let height = char_bmp.height;
                            let width = char_bmp.width;
                            if let Some(v_screen) = draw_ctx.v_screen.as_mut() {
                                if let Some(_bg_bmp) = draw_ctx.background_bitmap.as_ref() {
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

                            off_y += char_bmp.width + font.gap_width;
                        }
                        off_y += font.height;
                    }
                    off_y += font.height;
                }
            }
        }
        Ok(())
    }

    fn layout_text_line<'a>(&self, text: &'a str) -> LayoutResult<'a> {
        let mut line_width = 0;
        let word_width = 0;

        let word_boundary: Option<usize> = None;
        let mut end_index = text.len();

        let bytes = text.as_bytes();

        for (i, char_byte) in bytes.iter().enumerate() {
            let masked_char = char_byte & 0x7F;
            if masked_char == 0 || masked_char == b'\n' {
                end_index = i;
                break;
            }

            if let Some(ref fonts) = self.font {
                let char_bmp = &fonts.chars[masked_char as usize];
                if char_bmp.height == 0 {
                    continue;
                }

                let width = line_width + char_bmp.width + fonts.gap_width;
                if width > self.width {
                    // Nudge needed here
                    if let Some(boundary_idx) = word_boundary {
                        end_index = boundary_idx;
                        line_width = word_width;
                    } else {
                        end_index = i;
                    }
                    break;
                }
                line_width = width;
            }
        }

        let mut next_start = end_index;
        while next_start < bytes.len() && (bytes[next_start] & 0x7F) == b' ' {
            next_start += 1;
        }
        if next_start < bytes.len() && bytes[next_start] == b'\n' {
            next_start += 1;
        }

        LayoutResult {
            start: &text[..end_index],
            end: if next_start < text.len() {
                &text[next_start..]
            } else {
                ""
            },
            width: line_width,
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
        if self.font.is_some() || self.messages.front().is_none() {
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
                if let Some(front_msg) = self.messages.front() {
                    igTextWrapped(c"%s".as_ptr(), front_msg.text.as_ptr());
                }
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
    ) -> Result<TTextBox> {
        let mut instance = Self {
            offset_x: 0,
            offset_y: 0,
            width: 0,
            height: 0,
            timer: 0,
            bg_bmp: background_bitmap.clone(),
            font: score_state.MSG_FONTP.clone(),
            messages: VecDeque::new(),
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

struct LayoutResult<'a> {
    start: &'a str,
    end: &'a str,
    width: i32,
}
