use crate::gdrv::{ColorRgba, GdrvBitmap8};
use crate::loader;
use crate::state::control_state::ControlState;
use crate::state::loader_state::LoaderState;
use crate::state::pb_game_state::PbGameState;
use crate::state::pinball_state::PinballState;
use crate::state::render_state::RenderState;
use crate::state::sound_state::SoundState;
use crate::t_edge_manager::TEdgeManager;
use crate::t_pinball_component::IPinballComponent;
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
    pub control_state: &'a mut ControlState,
    pub loader_state: Option<&'a mut LoaderState>,
    pub sound_state: Option<&'a mut SoundState>,
    pub sound_stereo: bool,
    pub edge_manager: Option<&'a TEdgeManager>,
}

impl<'a> ComponentContext<'a> {
    pub fn from_state(state: &'a mut PinballState) -> ComponentContext<'a> {
        Self::from_parts_with_sound(
            &mut state.render_state,
            &state.pb_game_state,
            &mut state.control_state,
            state.timer_manager.clone(),
            &mut state.loader_state,
            &mut state.sound_state,
            *state.options_state.options.sound_stereo,
            state.pb_game_state.edge_manager.as_ref(),
        )
    }

    pub fn from_parts(
        render_state: &'a mut RenderState,
        pb_game_state: &'a PbGameState,
        control_state: &'a mut ControlState,
        timer_manager: Rc<RefCell<TimerManager>>,
    ) -> ComponentContext<'a> {
        Self {
            v_screen: &mut render_state.v_screen,
            current_palette: &pb_game_state.current_palette,
            time_ticks: pb_game_state.time_ticks,
            full_tilt_mode: pb_game_state.full_tilt_mode,
            background_bitmap: &render_state.background_bitmap,
            timer_manager,
            control_state,
            loader_state: None,
            sound_state: None,
            sound_stereo: false,
            edge_manager: pb_game_state.edge_manager.as_ref(),
        }
    }

    pub fn from_parts_with_sound(
        render_state: &'a mut RenderState,
        pb_game_state: &'a PbGameState,
        control_state: &'a mut ControlState,
        timer_manager: Rc<RefCell<TimerManager>>,
        loader_state: &'a mut LoaderState,
        sound_state: &'a mut SoundState,
        sound_stereo: bool,
        edge_manager: Option<&'a TEdgeManager>,
    ) -> ComponentContext<'a> {
        Self {
            v_screen: &mut render_state.v_screen,
            current_palette: &pb_game_state.current_palette,
            time_ticks: pb_game_state.time_ticks,
            full_tilt_mode: pb_game_state.full_tilt_mode,
            background_bitmap: &render_state.background_bitmap,
            timer_manager,
            control_state,
            loader_state: Some(loader_state),
            sound_state: Some(sound_state),
            sound_stereo,
            edge_manager,
        }
    }

    pub fn from_state_result(state: &'a mut PinballState) -> Result<ComponentContext<'a>> {
        Ok(Self {
            v_screen: &mut state.render_state.v_screen,
            current_palette: &state.pb_game_state.current_palette,
            time_ticks: state.pb_game_state.time_ticks,
            full_tilt_mode: state.pb_game_state.full_tilt_mode,
            background_bitmap: &state.render_state.background_bitmap,
            timer_manager: state.timer_manager.clone(),
            control_state: &mut state.control_state,
            loader_state: Some(&mut state.loader_state),
            sound_state: Some(&mut state.sound_state),
            sound_stereo: *state.options_state.options.sound_stereo,
            edge_manager: state.pb_game_state.edge_manager.as_ref(),
        })
    }
}

// Helpers!
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

    pub fn play_sound(
        &mut self,
        sound_index: i32,
        sound_source: Option<&dyn IPinballComponent>,
        info: &str,
    ) -> f32 {
        loader::play_sound(sound_index, sound_source, info, self)
    }
}
