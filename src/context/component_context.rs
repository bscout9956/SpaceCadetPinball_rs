use crate::gdrv::{ColorRgba, GdrvBitmap8};
use crate::state::pinball_state::PinballState;
use crate::timer::{TimerCallback, TimerManager};
use anyhow::Result;
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;

// This helper struct makes it so we don't have to pass 12930124312 arguments in draw/message/timer.set and others...
pub struct ComponentContext<'a> {
    pub v_screen: &'a mut Option<GdrvBitmap8>,
    pub current_palette: &'a [ColorRgba; 256],
    pub time_ticks: usize,
    pub full_tilt_mode: bool,
    pub background_bitmap: &'a Option<GdrvBitmap8>,
    pub timer_manager: Rc<RefCell<TimerManager>>,
}

impl<'a> ComponentContext<'a> {
    pub fn from_state(state: &'a mut PinballState) -> anyhow::Result<ComponentContext> {
        Ok(Self {
            v_screen: &mut state.render_state.v_screen,
            current_palette: &state.pb_game_state.current_palette,
            time_ticks: state.pb_game_state.time_ticks,
            full_tilt_mode: state.pb_game_state.full_tilt_mode,
            background_bitmap: &state.render_state.background_bitmap,
            timer_manager: state.timer_manager.clone(),
        })
    }
}

impl ComponentContext<'_> {
    pub fn set_timer(
        &self,
        delay: f32,
        caller: *mut c_void,
        callback: TimerCallback,
    ) -> Result<i32> {
        self.timer_manager
            .borrow_mut()
            .set(delay, caller, callback, self)
    }
}
