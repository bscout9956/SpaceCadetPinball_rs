#![allow(arithmetic_overflow)]
extern crate core;

use crate::embedded_data::load_controller_db;
use crate::errors::FullscreenError;
use crate::options::Menu::{FourPlayers, OnePlayer, ShowMenu, ThreePlayers, TwoPlayers};
use crate::options::{GameBindings, GameInput, InputTypes, Menu};
use crate::translations::{Msg, TranslationError};
use crate::utils::{SdlRendererPtr, SdlWindowPtr};
use dear_imgui_rs::sys::{
    ImGuiFocusRequestFlags_None, ImGuiMouseCursor_None, ImGuiSliderFlags_AlwaysClamp,
    ImGuiStyleVar_WindowMinSize, ImVec2_c, ImVec4, ImWchar, igBeginMainMenuBar, igBeginMenu, igEnd,
    igEndMainMenuBar, igEndMenu, igFocusWindow, igGetDrawData, igGetWindowSize, igMenuItem_Bool,
    igPopStyleVar, igPushStyleVar_Vec2, igRender, igSeparator, igSetMouseCursor, igSliderInt,
    igTextUnformatted,
};
use dear_imgui_rs::{ConfigFlags, Context, FontConfig, StyleColor, StyleVar, Ui};
use num_traits::FromPrimitive;
use sdl2::sys::SDL_EventType::{
    SDL_CONTROLLERBUTTONDOWN, SDL_CONTROLLERBUTTONUP, SDL_KEYDOWN, SDL_KEYUP, SDL_QUIT,
};
use sdl2::sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_ERROR;
use sdl2::sys::SDL_RendererFlags::{SDL_RENDERER_ACCELERATED, SDL_RENDERER_SOFTWARE};
use sdl2::sys::SDL_WindowFlags::{SDL_WINDOW_HIDDEN, SDL_WINDOW_RESIZABLE};
use sdl2::sys::SDL_bool::SDL_FALSE;
use sdl2::sys::mixer::{
    MIX_DEFAULT_FORMAT, MIX_DEFAULT_FREQUENCY, MIX_InitFlags_MIX_INIT_MID, MIX_MAJOR_VERSION,
    MIX_MINOR_VERSION, MIX_PATCHLEVEL, Mix_Init, Mix_OpenAudio,
};
use sdl2::sys::*;
use state::main_state::MainState;
use state::options_state::OptionsState;
use state::pinball_state::PinballState;
use std::env;
use std::error::Error;
use std::ffi::{CStr, CString, NulError, c_int};
use std::mem::MaybeUninit;
use std::ops::{Mul, Neg, Sub};
use std::path::PathBuf;
use std::process::exit;
use std::ptr::null;
use std::sync::{MutexGuard, PoisonError};
use thiserror::Error;

mod fullscrn;
mod gdrv;
mod group_data;
mod loader;
mod maths;
mod options;
mod score;
mod sound;
mod t_ball;
mod t_collision_component;
mod t_edge_segment;
mod t_flipper_edge;
mod t_pinball_component;
mod t_pinball_table;
mod translations;
mod zdrv;

pub mod control;
pub mod debug_overlay;
mod embedded_data;
mod errors;
pub mod high_score;
mod imgui_sdl;
pub mod message_code;
mod midi;
mod partman;
mod pb;
mod pinball_state;
pub mod proj;
mod render;
pub mod state;
pub mod t_demo;
mod t_edge_box;
mod t_edge_manager;
pub mod t_light;
mod t_light_group;
pub mod t_line;
pub mod t_table_layer;
pub mod t_textbox;
pub mod t_textbox_message;
pub mod text_array;
pub mod timer;
mod utils;

#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub struct Duration<const DEN: u64>(pub i64);
#[derive(PartialEq, Eq, Ord, PartialOrd, Clone, Copy)]
pub struct TimePoint<const DEN: u64>(Duration<DEN>);

impl<const DEN: u64> Duration<DEN> {
    pub fn count(&self) -> i64 {
        self.0
    }
}

impl<const DEN: u64> Sub for TimePoint<DEN> {
    type Output = Duration<DEN>;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration(self.0.count() - rhs.0.count())
    }
}

pub trait Clock {
    type Duration;
    type TimePoint;
    const IS_STEADY: bool;
    unsafe fn now() -> Self::TimePoint;
}

// TODO: Review these docs
/// A high-resolution, monotonic clock backed by SDL's performance counter.
///
/// This clock leverages `SDL_GetPerformanceCounter` and `SDL_GetPerformanceFrequency`
/// to provide steady, nanosecond-precision time measurement. It is highly suitable
/// for frame pacing, profiling, and precise delta-time calculations in an SDL context.
pub struct SdlPerformanceClock;

impl Clock for SdlPerformanceClock {
    /// The duration type representing differences between two time points.
    /// The generic parameter `1_000_000_000` indicates a nanosecond resolution.
    type Duration = Duration<1_000_000_000>;
    /// The absolute time point type returned by this clock.
    /// The generic parameter `1_000_000_000` indicates a nanosecond resolution.
    type TimePoint = TimePoint<1_000_000_000>;
    /// Indicates that this clock is monotonic.
    ///
    /// Because it is backed by hardware performance counters, time will continuously
    /// move forward and will not be affected by system clock adjustments (e.g., NTP syncing).
    const IS_STEADY: bool = true;

    /// Returns the current time point.
    ///
    /// This calculates the elapsed time in nanoseconds by separating the calculation
    /// into whole seconds and fractional seconds. This prevents integer overflow that
    /// could occur if the raw counter was multiplied by `1_000_000_000` before division.
    ///
    /// # Safety
    ///
    /// This function performs `unsafe` FFI calls to `SDL_GetPerformanceFrequency`
    /// and `SDL_GetPerformanceCounter`. The caller must ensure that the underlying
    /// SDL bindings allow these functions to be called in the current context
    /// (though generally, SDL allows querying the performance counter at any time).
    unsafe fn now() -> Self::TimePoint {
        let freq = unsafe { SDL_GetPerformanceFrequency() };
        let ctr = unsafe { SDL_GetPerformanceCounter() };
        let whole = (ctr / freq) * 1_000_000_000;
        let part = (ctr % freq) * 1_000_000_000 / freq;
        TimePoint(Duration((whole + part) as i64))
    }
}

impl Sub for Duration<1000000000> {
    type Output = Duration<1000000000>;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration(self.0 - rhs.0)
    }
}

impl Neg for Duration<1000000000> {
    type Output = Duration<1000000000>;

    fn neg(self) -> Self::Output {
        Duration(-self.0)
    }
}

pub struct SdlTickClock;
impl Clock for SdlTickClock {
    type Duration = Duration<1_000>;
    type TimePoint = TimePoint<1_000>;
    const IS_STEADY: bool = true;
    unsafe fn now() -> Self::TimePoint {
        let ticks = unsafe { SDL_GetTicks() };
        TimePoint(Duration(ticks as i64))
    }
}

pub struct WelfordState {
    pub mean: f64,
    pub m2: f64,
    pub count: i64,
}

impl WelfordState {
    fn new() -> Self {
        Self {
            mean: 0.005,
            m2: 0.0,
            count: 1,
        }
    }

    pub fn advance(&mut self, new_value: f64) {
        self.count += 1;
        let delta = new_value - self.mean;
        self.mean += delta / self.count as f64;
        self.m2 += delta * (new_value - self.mean);
    }

    pub fn get_std_dev(&self) -> f64 {
        f64::sqrt(self.m2 / (self.count - 1) as f64)
    }
}

pub const VERSION: &str = "1.0 DEV";

pub type DurationMs = f64;

#[derive(Debug, Error)]
pub enum MainError {
    #[error("Renderer hasn't been assigned, is none")]
    NoneRendererError,

    #[error("Unable to lock Mutex for retrieving the SDL_Renderer on main")]
    MutexError(#[from] PoisonError<MutexGuard<'static, Option<SdlRendererPtr>>>),
    #[error("Failed to lock Mutex")]
    LockGeneric,
}

unsafe fn render_frame_time_dialog() {
    unsafe {
        igPushStyleVar_Vec2(ImGuiStyleVar_WindowMinSize, ImVec2_c::new(300.0, 70.0));

        igEnd();
        igPopStyleVar(0);
    }
}

fn hybrid_sleep(mut sleep_target: Duration<1000000000>, main_state: &mut MainState) {
    let std_dev_factor = 0.5f64;

    // This nice concept is from https://blat-blatnik.github.io/computerBear/making-accurate-sleep-function/
    // Sacrifices some CPU time for smaller frame time jitter
    while sleep_target > main_state.spin_threshold {
        let start = unsafe { SdlPerformanceClock::now() };
        std::thread::sleep(std::time::Duration::from_nanos(start.0.count() as u64)); // TODO: Is this correct?
        let end = unsafe { SdlPerformanceClock::now() };

        let actual_duration = end - start;
        sleep_target = sleep_target - actual_duration;
        main_state
            .sleep_state
            .advance(actual_duration.count() as f64);
        main_state.spin_threshold = Duration(
            (main_state.sleep_state.mean + main_state.sleep_state.get_std_dev() * std_dev_factor)
                as i64,
        );
    }

    unsafe {
        // spin lock
        let start = SdlPerformanceClock::now();
        while SdlPerformanceClock::now() - start < sleep_target {}
    }
}

fn imgui_menu_item_w_shortcut(
    binding: GameBindings,
    selected: Option<bool>,
    options_state: &mut OptionsState,
) {
    let idx = (binding as usize)
        .checked_sub(1)
        .expect("Invalid binding index");

    let key_def = options_state
        .options
        .control_options
        .get(idx)
        .expect("Binding not found in control options");

    let select = selected.unwrap_or(false);
    let shortcut_cstr = CString::new(key_def.get_shortcut_description()).unwrap();
    let desc = pb::get_rc_string_cstring(key_def.description).unwrap();
    unsafe {
        if igMenuItem_Bool(desc.as_ptr(), shortcut_cstr.as_ptr(), select, true) {
            handle_game_binding(binding, false);
        }
    };
}

fn handle_game_binding(bind: GameBindings, p1: bool) {
    //todo implement me
}

#[derive(Error, Debug)]
pub enum MainLoopError {
    #[error("Failed to lock Mutex")]
    MutexLock,
    #[error(transparent)]
    NulError(#[from] NulError),
    #[error("There is no MainWindow to attach to...")]
    NullWindow,
    #[error(transparent)]
    FullScreen(#[from] FullscreenError),
    #[error(transparent)]
    Translation(#[from] TranslationError),
}

impl Mul<Duration<1000000000>> for i32 {
    type Output = Duration<1000000000>;

    fn mul(self, rhs: Duration<1000000000>) -> Self::Output {
        Duration(self as i64 * rhs.0)
    }
}

fn main_loop(
    imgui_context: &mut Context,
    pb_state: &mut PinballState,
) -> Result<(), MainLoopError> {
    pb_state.main_state.b_quit = false;

    let mut update_count: usize = 0;
    let mut frame_counter: usize = 0;

    let mut frame_start = unsafe { SdlPerformanceClock::now() };
    let mut prev_time = frame_start;

    let mut update_to_frame_counter = 0.0;
    let mut sleep_remainder: Duration<1_000_000_000> = Duration(0);

    let mut frame_duration = pb_state.main_state.target_frametime;

    println!("Entering main loop, loop");
    loop {
        if pb_state.main_state.disp_frame_rate {
            let cur_time = unsafe { SdlPerformanceClock::now() };

            if (cur_time - prev_time) > Duration(1_000_000_000) {
                let elapsed_sec = (cur_time - prev_time).count() as f64 / 1_000_000_000f64;

                let title = format!(
                    "Updates/sec = {:02.2} Frames/sec = {:02.2} ",
                    update_count as f64 / elapsed_sec,
                    frame_counter as f64 / elapsed_sec
                );
                let c_str_title = CString::new(title.clone())?;

                if let Some(window) = pb_state.main_state.main_window.as_ref() {
                    unsafe {
                        SDL_SetWindowTitle(window.0, c_str_title.as_ptr());
                    };
                } else {
                    return Err(MainLoopError::NullWindow);
                }

                (&mut pb_state.main_state).update_fps_details(&title);
                update_count = 0;
                frame_counter = update_count;
                prev_time = cur_time;
            }
        }

        if !process_window_messages(imgui_context, pb_state)? || pb_state.main_state.b_quit {
            break;
        }

        if pb_state.main_state.has_focus && pb_state.main_state.mouse_down {
            let mut x = 0;
            let mut y = 0;
            let mut w = 0;
            let mut h = 0;
            unsafe {
                SDL_GetMouseState(&mut x, &mut y);

                if let Some(window) = pb_state.main_state.main_window.as_ref() {
                    SDL_GetWindowSize(window.0, &mut w, &mut h);
                } else {
                    return Err(MainLoopError::NullWindow);
                }
            }
            let dx = (pb_state.main_state.last_mouse_x - x) as f32 / w as f32;
            let dy = (y - pb_state.main_state.last_mouse_y) as f32 / h as f32;
            pb::ball_set(dx, dy, &mut pb_state.pb_game_state);

            // Original creates continuous mouse movement with mouse capture.
            // Alternative solution: mouse warp at window edges.
            let mut x_mod: i32 = 0;
            let mut y_mod: i32 = 0;

            if x == 0 || x >= (w - 1) {
                x_mod = w - 2;
            }
            if y == 0 || y >= (h - 1) {
                y_mod = h - 2;
            }

            unsafe {
                if (x_mod != 0 || y_mod != 0) {
                    x = i32::abs(x - x_mod);
                    y = i32::abs(y - y_mod);
                    if let Some(window) = pb_state.main_state.main_window.as_ref() {
                        SDL_WarpMouseInWindow(window.0, x, y);
                    }
                }
            }

            pb_state.main_state.update_mouse_xy(x, y);
        }

        // Scope to avoid repetition, Rust usually doesn't like long-lived scopes but in this case it helps
        {
            let main_state = &mut pb_state.main_state;
            if !main_state.single_step && !main_state.no_time_loss {
                let dt = frame_duration.count() as f32;
                pb::frame(dt, &mut pb_state.pb_game_state);

                if main_state.disp_gr_history {
                    let target_size = (*pb_state.options_state.options.updates_per_second as f32
                        * main_state.gfr_window) as usize;
                    if main_state.gfr_display.len() != target_size {
                        main_state
                            .gfr_display
                            .resize(target_size, main_state.target_frametime.count() as f32);
                        main_state.gfr_offset = 0;
                    }
                    main_state.gfr_display[main_state.gfr_offset as usize] = dt;
                    main_state.gfr_offset =
                        (main_state.gfr_offset + 1) % main_state.gfr_display.len() as u32;
                }
                update_count += 1
            }
            main_state.no_time_loss = false;
        }

        if update_to_frame_counter >= (&mut pb_state.main_state).update_to_frame_ratio {
            if *pb_state.options_state.options.hide_cursor
                && (&mut pb_state.main_state).cursor_idle_counter <= 0
            {
                unsafe {
                    igSetMouseCursor(ImGuiMouseCursor_None);
                };
            }

            unsafe {
                imgui_sdl::impl_sdl2_new_frame(imgui_context.io_mut(), pb_state);
                imgui_sdl::impl_sdl2_renderer_new_frame(imgui_context);
                let ui = imgui_context.frame();
                render_ui(ui, pb_state)?;
                if let Some(renderer) = pb_state.main_state.renderer.as_ref() {
                    SDL_RenderClear(renderer.0);
                    SDL_RenderFillRect(renderer.0, null());
                }
                render::present_v_screen(pb_state);
                igRender();
                let draw_data = igGetDrawData();
                imgui_sdl::renderer::render_draw_data(imgui_context.io_mut(), draw_data);

                if let Some(renderer) = pb_state.main_state.renderer.as_mut() {
                    SDL_RenderPresent(renderer.0);
                } else {
                    panic!("No renderer")
                }

                frame_counter += 1;
                update_to_frame_counter -= pb_state.main_state.update_to_frame_ratio;
            }
        }

        unsafe {
            let sdl_error = SDL_GetError();
            if !sdl_error.is_null() || !pb_state.main_state.prev_sdl_error.is_empty() {
                if !sdl_error.is_null() {
                    SDL_ClearError();
                }

                let sdl_error_str = CStr::from_ptr(sdl_error);
                let sdl_error_string = sdl_error_str.to_string_lossy().to_string();
                if sdl_error_string != pb_state.main_state.prev_sdl_error {
                    println!("SDL error: {}", sdl_error_string);
                    println!(
                        "SDL error (main_state): {}",
                        pb_state.main_state.prev_sdl_error
                    );

                    pb_state.main_state.prev_sdl_error = String::from(&sdl_error_string);
                    if pb_state.main_state.prev_sdl_error_count > 0 {
                        println!(
                            "SDL Error: ^ Previous error repeated {} times",
                            pb_state.main_state.prev_sdl_error_count + 1
                        );
                        pb_state.main_state.prev_sdl_error_count = 0;
                    }

                    if !sdl_error.is_null() {
                        println!("SDL error: {}", &sdl_error_string);
                    }
                } else {
                    pb_state.main_state.prev_sdl_error_count += 1;
                }
            }

            let update_end = SdlPerformanceClock::now();
            let target_time_delta =
                pb_state.main_state.target_frametime - (update_end - frame_start) - sleep_remainder;

            let frame_end;

            if target_time_delta.count() > 0
                && !*pb_state.options_state.options.uncapped_updates_per_second
            {
                if *pb_state.options_state.options.hybrid_sleep {
                    hybrid_sleep(target_time_delta, &mut pb_state.main_state);
                } else {
                    let ns = target_time_delta.count().max(0) as u64;
                    let secs = ns / 1_000_000_000;
                    let nanos = (ns % 1_000_000_000) as u32;
                    std::thread::sleep(std::time::Duration::new(secs, nanos));
                }
                frame_end = SdlPerformanceClock::now();
            } else {
                frame_end = update_end;
            }

            // Limit duration to 2 * target time
            sleep_remainder = utils::clamp(
                &((frame_end - update_end) - target_time_delta),
                &(-pb_state.main_state.target_frametime),
                &pb_state.main_state.target_frametime,
            );
            frame_duration = std::cmp::min(
                frame_end - frame_start,
                2 * pb_state.main_state.target_frametime,
            );
            frame_start = frame_end;
            update_to_frame_counter += 1.0;

            pb_state.main_state.cursor_idle_counter = std::cmp::max(
                pb_state.main_state.cursor_idle_counter - frame_duration.count(),
                0,
            );
        }
    }

    if pb_state.main_state.prev_sdl_error_count > 0 {
        println!(
            "SDL Error: ^ Previous error repeated {} times",
            pb_state.main_state.prev_sdl_error_count
        );
    }

    Ok(())
}

unsafe fn create_options_menu(state: &mut PinballState) -> Result<(), MainLoopError> {
    unsafe {
        let menu_string = pb::get_rc_string_cstring(Msg::Menu1Options)?;
        if igBeginMenu(menu_string.as_ptr(), true) {
            imgui_menu_item_w_shortcut(
                GameBindings::ToggleMenuDisplay,
                Some(*state.options_state.options.show_menu),
                &mut state.options_state,
            );
            imgui_menu_item_w_shortcut(
                GameBindings::ToggleFullScreen,
                Some(*state.options_state.options.full_screen),
                &mut state.options_state,
            );

            let select_player_string = pb::get_rc_string_cstring(Msg::Menu1SelectPlayers)?;
            if igBeginMenu(select_player_string.as_ptr(), true) {
                let one_player_string = pb::get_rc_string_cstring(Msg::MENU1_1PLAYER)?;
                if igMenuItem_Bool(
                    one_player_string.as_ptr(),
                    null(),
                    *state.options_state.options.players == 1,
                    true,
                ) {
                    options::toggle(OnePlayer, state);
                    // TODO: new_game();
                }

                if igMenuItem_Bool(
                    pb::get_rc_string_cstring(Msg::MENU1_2PLAYERS)?.as_ptr(),
                    null(),
                    *state.options_state.options.players == 2,
                    true,
                ) {
                    options::toggle(TwoPlayers, state);
                    // TODO: new_game();
                }

                if igMenuItem_Bool(
                    pb::get_rc_string_cstring(Msg::MENU1_3PLAYERS)?.as_ptr(),
                    null(),
                    *state.options_state.options.players == 3,
                    true,
                ) {
                    options::toggle(ThreePlayers, state);
                    // TODO: new_game();
                }

                if igMenuItem_Bool(
                    pb::get_rc_string_cstring(Msg::MENU1_4PLAYERS)?.as_ptr(),
                    null(),
                    *state.options_state.options.players == 4,
                    true,
                ) {
                    options::toggle(FourPlayers, state);
                    // TODO: new_game();
                }
                igEndMenu();
            }
            imgui_menu_item_w_shortcut(
                GameBindings::ShowControlDialog,
                Option::None,
                &mut state.options_state,
            );

            if igBeginMenu(c"Language".as_ptr(), true) {
                let current_language = translations::get_current_language().unwrap();
                for item in translations::LANGUAGES {
                    let name = CString::new(item.display_name.to_string())?;
                    if igMenuItem_Bool(
                        name.as_ptr(),
                        null(),
                        current_language.language == item.language,
                        true,
                    ) {
                        if current_language.language != item.language {
                            translations::set_current_language(item.short_name);
                            //restart();
                        }
                    }
                }

                igEndMenu();
            }

            igSeparator();

            create_audio_menu(state);
            // create_graphics_menu();
            create_resolution_menu(state);
            create_game_data_menu(state);

            igSeparator();

            if igMenuItem_Bool(c"Reset All Options".as_ptr(), null(), false, true) {
                //TODO, needs io
                // options::reset_all_options()
                //restart();
            }
            igEndMenu();
        }

        Ok(())
    }
}

unsafe fn create_resolution_menu(state: &mut PinballState) -> Result<(), MainLoopError> {
    unsafe {
        let table_res_string = pb::get_rc_string_cstring(Msg::Menu1TableResolution)?;
        if igBeginMenu(table_res_string.as_ptr(), true) {
            let mut resolution_string_id = Msg::Menu1UseMaxResolution640x480;

            match fullscrn::get_max_resolution(&mut state.pb_game_state) {
                0 => {
                    resolution_string_id = Msg::Menu1UseMaxResolution640x480;
                }
                1 => {
                    resolution_string_id = Msg::Menu1UseMaxResolution800x600;
                }
                2 => {
                    resolution_string_id = Msg::Menu1UseMaxResolution1024x768;
                }
                _ => {}
            }

            let max_res_text = pb::get_rc_string_cstring(resolution_string_id)?;
            if igMenuItem_Bool(
                max_res_text.as_ptr(),
                null(),
                *state.options_state.options.resolution == -1,
                true,
            ) {
                options::toggle(Menu::MaximumResolution, state);
            }

            for i in 0..=fullscrn::get_max_resolution(&mut state.pb_game_state) {
                let res = &state.fullscrn_state.resolution_array[i as usize];
                let text = format!("{} x {}", res.screen_width, res.screen_height);
                let cstr = CString::new(text)?;
                if igMenuItem_Bool(
                    cstr.as_ptr(),
                    null(),
                    *state.options_state.options.resolution == i,
                    true,
                ) {
                    match i {
                        0 => {
                            options::toggle(Menu::R640x480, state);
                        }
                        1 => {
                            options::toggle(Menu::R800x600, state);
                        }
                        2 => {
                            options::toggle(Menu::R1024x768, state);
                        }
                        _ => {
                            options::toggle(Menu::R640x480, state);
                        }
                    };
                }
            }
            igEndMenu();
        }
    }
    Ok(())
}

unsafe fn create_audio_menu(state: &mut PinballState) {
    unsafe {
        if igBeginMenu(c"Audio".as_ptr(), true) {
            imgui_menu_item_w_shortcut(
                GameBindings::ToggleSounds,
                Some(*state.options_state.options.sounds),
                &mut state.options_state,
            );

            if igMenuItem_Bool(
                c"Stereo Sound Effects".as_ptr(),
                null(),
                *state.options_state.options.sound_stereo,
                true,
            ) {
                options::toggle(Menu::SoundStereo, state);
            }
            igTextUnformatted(c"Sound Volume".as_ptr(), c"".as_ptr());
            if igSliderInt(
                c"##Sound Volume".as_ptr(),
                &raw mut state.options_state.options.sound_volume.value,
                options::MIN_VOLUME,
                options::MAX_VOLUME,
                c"%d".as_ptr(),
                ImGuiSliderFlags_AlwaysClamp,
            ) {
                sound::set_volume(*state.options_state.options.sound_volume);
            }
            igTextUnformatted(c"Sound Channels".as_ptr(), c"".as_ptr());
            if igSliderInt(
                c"##Sound Channels".as_ptr(),
                &raw mut state.options_state.options.sound_channels.value,
                options::MIN_SOUND_CHANNELS,
                options::MAX_SOUND_CHANNELS,
                c"%d".as_ptr(),
                ImGuiSliderFlags_AlwaysClamp,
            ) {
                sound::set_channels(*state.options_state.options.sound_channels);
            }
            igSeparator();

            imgui_menu_item_w_shortcut(
                GameBindings::ToggleMusic,
                Some(*state.options_state.options.music),
                &mut state.options_state,
            );
            igTextUnformatted(c"Music Volume".as_ptr(), c"".as_ptr());
            if igSliderInt(
                c"##Music Volume".as_ptr(),
                &raw mut state.options_state.options.music_volume.value,
                options::MIN_VOLUME,
                options::MAX_VOLUME,
                c"%d".as_ptr(),
                ImGuiSliderFlags_AlwaysClamp,
            ) {
                midi::set_volume(*state.options_state.options.music_volume);
            }
            igEndMenu();
        }
    }
}

unsafe fn create_game_data_menu(state: &mut PinballState) {
    unsafe {
        if igBeginMenu(c"Game Data".as_ptr(), true) {
            if igMenuItem_Bool(
                c"Prefer 3DPB Data".as_ptr(),
                null(),
                *state.options_state.options.prefer_3dpb_game_data,
                true,
            ) {
                options::toggle(Menu::Prefer3DPBGameData, state);
            }
            igEndMenu();
        }
    }
}

unsafe fn create_main_menu_bar(state: &mut PinballState) -> Result<(), MainLoopError> {
    unsafe {
        if *state.options_state.options.show_menu && igBeginMainMenuBar() {
            let current_menu_height = igGetWindowSize().y as i32;
            if state.main_state.main_menu_height != current_menu_height {
                // Get the height of the main menu bar and update screen coordinates
                state.main_state.main_menu_height = current_menu_height;
                fullscrn::window_size_changed(state)?;
            }

            // create_game_menu();
            create_options_menu(state);
            // create_help_menu();

            if state.main_state.disp_frame_rate && !state.main_state.fps_details.is_empty() {
                let cstr = CString::new(state.main_state.fps_details.as_str())?;
                if igBeginMenu(cstr.as_ptr(), true) {
                    igEndMenu();
                }
            }

            igEndMainMenuBar();
        }
    }

    Ok(())
}

unsafe fn render_ui(ui: &mut Ui, state: &mut PinballState) -> Result<(), MainLoopError> {
    unsafe {
        let _menu_bar_bg =
            ui.push_style_color(StyleColor::MenuBarBg, ImVec4::new(0.0, 0.0, 0.0, 0.0));
        let _window_bg = ui.push_style_color(StyleColor::WindowBg, ImVec4::new(0.0, 0.0, 0.0, 0.0));
        let _border_var = ui.push_style_var(StyleVar::WindowBorderSize(0.0));

        if !(state.options_state.options.show_menu.value) && igBeginMainMenuBar() {
            let menu_string = "Menu".to_string();
            let cstr_menu = CString::new(menu_string)?;

            if igMenuItem_Bool(cstr_menu.as_ptr(), null(), false, true) {
                options::toggle(ShowMenu, state);
                igFocusWindow(std::ptr::null_mut(), ImGuiFocusRequestFlags_None);
            }

            igEndMainMenuBar();
        }

        create_main_menu_bar(state);

        // render_dialogs();

        // if *state.options_state.options.show_menu && igBeginMainMenuBar() {
        //     let current_menu_height = igGetWindowSize().y as i32;
        //     if state.main_state.main_menu_height != current_menu_height {
        //         state.main_state.main_menu_height = current_menu_height;
        //         fullscrn::window_size_changed(state)?;
        //     }
        //
        //     let menu_game_string = pb::get_rc_string_cstring(Msg::Menu1Game)?;
        //     if igBeginMenu(menu_game_string.as_ptr(), true) {
        //         imgui_menu_item_w_shortcut(
        //             GameBindings::NewGame,
        //             Option::None,
        //             &mut state.options_state,
        //         );
        //
        //         let launch_ball_string = pb::get_rc_string_cstring(Msg::Menu1LaunchBall)?;
        //         if igMenuItem_Bool(
        //             launch_ball_string.as_ptr(),
        //             null(),
        //             false,
        //             state.main_state.launch_ball_enabled,
        //         ) {
        //             end_pause(&mut state.main_state);
        //             pb::launch_ball();
        //         }
        //         imgui_menu_item_w_shortcut(
        //             GameBindings::TogglePause,
        //             Option::None,
        //             &mut state.options_state,
        //         );
        //         igSeparator();
        //
        //         let high_score_string = pb::get_rc_string_cstring(Msg::Menu1HighScores)?;
        //         if igMenuItem_Bool(
        //             high_score_string.as_ptr(),
        //             null(),
        //             false,
        //             state.main_state.high_scores_enabled,
        //         ) {
        //             pause(false, &mut state.main_state);
        //             pb::high_scores(&mut state.high_score_state);
        //         }
        //
        //         if igMenuItem_Bool(
        //             pb::get_rc_string_cstring(Msg::Menu1Demo)?.as_ptr(),
        //             null(),
        //             state.main_state.demo_active,
        //             true,
        //         ) {
        //             end_pause(&mut state.main_state);
        //             pb::toggle_demo();
        //         }
        //
        //         imgui_menu_item_w_shortcut(
        //             GameBindings::Exit,
        //             Option::None,
        //             &mut state.options_state,
        //         );
        //         igEndMenu();
        //     }
        //
        //
        //
        //     imgui_menu_item_w_shortcut(
        //         GameBindings::ShowControlDialog,
        //         Option::None,
        //         &mut state.options_state,
        //     );
        //
        //     if igBeginMenu(c"Language".as_ptr(), true) {
        //         let current_language = translations::get_current_language().unwrap();
        //         for item in translations::LANGUAGES {
        //             let name = CString::new(item.display_name.to_string())?;
        //             if igMenuItem_Bool(
        //                 name.as_ptr(),
        //                 null(),
        //                 current_language.language == item.language,
        //                 true,
        //             ) {
        //                 translations::set_current_language(item.short_name);
        //                 //restart();
        //             }
        //         }
        //
        //         igEndMenu();
        //     }
        //     igSeparator();
        //
        //     if state.main_state.disp_frame_rate && !state.main_state.fps_details.is_empty() {
        //         let cstr = CString::new(state.main_state.fps_details.as_str());
        //         if igBeginMenu(cstr?.as_ptr(), true) {
        //             igEndMenu();
        //         }
        //     }
        //     igEndMainMenuBar();
        // }

        // Print game texts on the sidebar
        //TODO: gdrv::gr_text_draw_ttext_in_box(&mut state.render_state, &mut state.pb_game_state, ui);
    }
    Ok(())
}

pub fn restart(main_state: &mut MainState) {
    main_state.restart = true;
    unsafe {
        let mut event = SDL_Event {
            type_: SDL_QUIT as u32,
        };
        let event_ptr = &raw mut event;
        SDL_PushEvent(event_ptr);
    }
}

fn pause(toggle: bool, main_state: &mut MainState) {
    if toggle || !main_state.single_step {
        pb::pause_continue(main_state);
        main_state.no_time_loss = true;
    }
}

fn process_window_messages(
    imgui_context: &mut Context,
    state: &mut PinballState,
) -> Result<bool, MainLoopError> {
    let mut idle_wait = 0i64;
    let mut event = MaybeUninit::<SDL_Event>::uninit();

    if state.main_state.has_focus {
        idle_wait = state.main_state.target_frametime.count();
        unsafe {
            while SDL_PollEvent(event.as_mut_ptr()) > 0 {
                if event_handler(event.as_mut_ptr(), imgui_context, state)? == false {
                    return Ok(false);
                }
            }
        }

        return Ok(true);
    }

    // Progressively wait longer when transitioning to idle
    idle_wait = i64::min(idle_wait + state.main_state.target_frametime.0, 500);
    unsafe {
        if SDL_WaitEventTimeout(event.as_mut_ptr(), idle_wait as c_int) > 0 {
            idle_wait = state.main_state.target_frametime.count();
            return event_handler(event.as_mut_ptr(), imgui_context, state);
        }
    }
    Ok(true)
}

unsafe fn event_handler(
    event: *mut SDL_Event,
    context: &mut Context,
    state: &mut PinballState,
) -> Result<bool, MainLoopError> {
    let mut input_down = false;

    unsafe {
        // Evaluate the cast in an if-statement instead of a pattern match
        if (*event).type_ == SDL_KEYDOWN as u32
            || (*event).type_ == SDL_EventType::SDL_MOUSEBUTTONDOWN as u32
            || (*event).type_ == SDL_CONTROLLERBUTTONDOWN as u32
        {
            input_down = true;
        }
    }

    if state.options_state.control_waiting_for_input.is_none() || !input_down {
        imgui_sdl::impl_sdl2_process_event(context, event);
    }

    let mouse_event: bool;

    unsafe {
        if (*event).type_ == SDL_EventType::SDL_MOUSEMOTION as u32
            || (*event).type_ == SDL_EventType::SDL_MOUSEBUTTONDOWN as u32
            || (*event).type_ == SDL_EventType::SDL_MOUSEBUTTONUP as u32
            || (*event).type_ == SDL_EventType::SDL_MOUSEWHEEL as u32
        {
            state.main_state.cursor_idle_counter = 1000;
            mouse_event = true;
        } else {
            mouse_event = false;
        }
    }

    let io = context.io_mut();

    if io.want_capture_mouse() && state.options_state.control_waiting_for_input.is_none() {
        if state.main_state.mouse_down == true {
            state.main_state.mouse_down = false;
            if let Some(window) = state.main_state.main_window.as_ref() {
                unsafe {
                    SDL_SetWindowGrab(window.0, SDL_FALSE);
                }
            }
        }

        if mouse_event {
            return Ok(true);
        }
    }

    if io.want_capture_keyboard() && state.options_state.control_waiting_for_input.is_none() {
        unsafe {
            if (*event).type_ == SDL_KEYUP as u32
                || (*event).type_ == SDL_KEYDOWN as u32
                || (*event).type_ == SDL_CONTROLLERBUTTONDOWN as u32
                || (*event).type_ == SDL_CONTROLLERBUTTONUP as u32
            {
                return Ok(true);
            }
        }
    }

    unsafe {
        if (*event).type_ == SDL_QUIT as u32 {
            end_pause(&mut state.main_state);

            state.main_state.b_quit = true;
            fullscrn::shutdown(&mut state.fullscrn_state, &mut state.main_state.main_window);
            state.main_state.return_value = 0;
            return Ok(false);
        }
        if (*event).type_ == SDL_KEYUP as u32 {
            pb::input_up(
                GameInput::new(InputTypes::Keyboard, (*event).key.keysym.sym),
                &mut state.main_state,
                &mut state.pb_game_state,
            );
        }
    }
    Ok(true)
}

fn end_pause(main_state: &mut MainState) {
    if main_state.single_step == true {
        pb::pause_continue(main_state);
        main_state.no_time_loss = true;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut state = PinballState::new();

    println!("Game version: {}", VERSION);
    let args: Vec<String> = env::args().collect();
    println!("Command line: {:?}", args);
    print!(
        "Compiled with: SDL {}.{}.{}",
        SDL_MAJOR_VERSION, SDL_MINOR_VERSION, SDL_PATCHLEVEL
    );
    print!(
        " SDL_mixer {}.{}.{};",
        MIX_MAJOR_VERSION, MIX_MINOR_VERSION, MIX_PATCHLEVEL
    );
    println!(" ImGui TODO");

    let sdl_context = sdl2::init()?;
    unsafe {
        SDL_SetMainReady();
        if (SDL_Init(
            SDL_INIT_TIMER
                | SDL_INIT_AUDIO
                | SDL_INIT_VIDEO
                | SDL_INIT_EVENTS
                | SDL_INIT_JOYSTICK
                | SDL_INIT_GAMECONTROLLER,
        ) < 0)
        {
            pb::show_message_box_cstr_message(
                SDL_MESSAGEBOX_ERROR,
                "Could not initialize SDL2",
                SDL_GetError(),
                &state.main_state.main_window,
            );
            println!("OOPS!! No init, closing");
            exit(1);
        }
    }

    let quick_flag = args.iter().any(|arg| arg.contains("-quick"));
    (&mut state.pb_game_state).quick_flag = quick_flag;

    unsafe {
        println!("Creating window");
        let rc_string = pb::get_rc_string(Msg::STRING139)?;
        let c_string = CString::new(rc_string)?;

        let window = SDL_CreateWindow(
            c_string.as_ptr(),
            20,
            50,
            800,
            556,
            SDL_WINDOW_HIDDEN as u32 | SDL_WINDOW_RESIZABLE as u32,
        );

        let mut main_window: Option<SdlWindowPtr> = Option::None;
        if !window.is_null() {
            main_window = Some(SdlWindowPtr(window));
        }

        state.main_state.main_window = main_window;

        if state.main_state.main_window.is_none() {
            pb::show_message_box_cstr_message(
                SDL_MESSAGEBOX_ERROR,
                "Could not create window",
                SDL_GetError(),
                &state.main_state.main_window,
            );
            println!("Could not create window");
            exit(1);
        }

        let sw_offset_flag = args.iter().any(|arg| arg.contains("-sw"));
        for i in sw_offset_flag as i32..2 {
            println!("Offset {}", i);
            let flags = if i == 0 {
                print!("Using HW accel");
                SDL_RENDERER_ACCELERATED
            } else {
                println!("Using software");
                SDL_RENDERER_SOFTWARE
            };
            let renderer: *mut SDL_Renderer = SDL_CreateRenderer(window, -1, flags as u32);

            if !renderer.is_null() {
                (&mut state.main_state).renderer = Some(SdlRendererPtr(renderer));
                println!("Renderer successfully created and assigned.");
                break;
            }
        }

        if (&mut state.main_state).renderer.is_none() {
            pb::show_message_box_cstr_message(
                SDL_MESSAGEBOX_ERROR,
                "Could not create renderer",
                SDL_GetError(),
                &state.main_state.main_window,
            );
            println!("Could not create renderer, is null");
            exit(1);
        }

        let mut renderer_info: SDL_RendererInfo = std::mem::zeroed();

        if let Some(renderer_ptr) = (&mut state.main_state).renderer.as_ref() {
            let result = SDL_GetRendererInfo(renderer_ptr.0, &mut renderer_info);

            if result != 0 {
                println!("Error getting renderer information");
            } else {
                println!(
                    "Using SDL Renderer: {}",
                    CStr::from_ptr(renderer_info.name).to_str()?
                );
            }

            SDL_SetRenderDrawColor(renderer_ptr.0, 0, 0, 0, 255);
        }

        SDL_SetHint(
            SDL_HINT_RENDER_SCALE_QUALITY.as_ptr() as *const i8,
            c"nearest".as_ptr(),
        );

        let pref_path = SDL_GetPrefPath(c"".as_ptr(), c"SpaceCadetPinball".as_ptr());
        let base_path = SDL_GetBasePath();

        let mut mix_opened = false;
        let no_audio = env::args().any(|arg| arg.contains("-noaudio"));
        if !no_audio {
            println!("Audio enabled.");
            if ((Mix_Init(MIX_InitFlags_MIX_INIT_MID as c_int)
                & MIX_InitFlags_MIX_INIT_MID as c_int)
                == 0)
            {
                println!(
                    "Could not initialize SDL MIDI, music might not work.\nSDL Error:{}",
                    CStr::from_ptr(SDL_GetError()).to_str()?
                );
                SDL_ClearError();
            }
            if (Mix_OpenAudio(
                MIX_DEFAULT_FREQUENCY as c_int,
                MIX_DEFAULT_FORMAT as u16,
                2,
                1024,
            ) != 0)
            {
                println!(
                    "Could not open audio device, continuing without audio.\nSDL Error:{}",
                    CStr::from_ptr(SDL_GetError()).to_str()?
                );
                SDL_ClearError();
            } else {
                println!("Mix opened!");
                mix_opened = true;
            }
        }

        // Load SDL Game Controller definitions from DB
        // This is more Rust idiomatic because all solutions I've tried were a complete nightmare...
        // Including trying to mimic the original one
        match load_controller_db(&sdl_context) {
            Ok(_) => {
                println!("Loaded controller.");
            }
            Err(e) => {
                println!("Error loading controller: {}", e);
            }
        }

        let mut reset_all_options = env::args().any(|arg| arg.contains("-reset"));

        println!("Entering loop");
        loop {
            state.main_state.restart = false;

            // ImGUi Init
            let mut imgui_context = Context::create();
            let io = imgui_context.io_mut();
            let mut cfg_flags = io.config_flags();

            let font_cfg = FontConfig::new().oversample_v(2).oversample_h(4);

            let pref_path_string =
                CStr::from_ptr(pref_path).to_string_lossy().into_owned() + "imgui_pb.ini";
            let ini_path = PathBuf::from(pref_path_string);

            imgui_context.set_ini_filename(Some(ini_path))?;

            // First option initialization step: just load settings from .ini. Needs ImGui context.
            options::init_primary(
                imgui_context.io_mut(),
                &mut state.main_state,
                &mut state.options_state,
            );
            if reset_all_options {
                reset_all_options = false;
                options::reset_all_options(
                    imgui_context.io_mut(),
                    &mut state.main_state,
                    &mut state.options_state,
                );
            }

            let font_file_name = &state.options_state.options.font_file_name.value;
            println!("Using FontFileName = {}", font_file_name);

            if !font_file_name.is_empty() {
                let mut fonts = imgui_context.fonts();
                let ranges = build_glyph_ranges_from_translations();

                if std::fs::File::open(font_file_name).is_ok() {
                    let custom_font = fonts.add_font_from_file_ttf(
                        font_file_name,
                        13.0,
                        Some(&font_cfg),
                        Some(&ranges),
                    );
                    if custom_font.is_none() {
                        println!(
                            "Could not load font {}, falling back to default",
                            font_file_name
                        );
                        imgui_context.fonts().add_font_default(Some(&font_cfg));
                    }
                } else {
                    println!(
                        "Font file not found: {}, using embedded font.",
                        font_file_name
                    );
                    imgui_context.fonts().add_font_default(Some(&font_cfg));
                }
            } else {
                imgui_context.fonts().add_font_default(Some(&font_cfg));
            }

            imgui_context.fonts().build();

            println!("Initializing IMGUI_SDL");
            if let Some(renderer) = state.main_state.renderer.as_ref() {
                imgui_sdl::renderer::init(&mut imgui_context, renderer.0);
                imgui_sdl::init_for_sdl_renderer(&mut imgui_context, window, renderer.0);
            }

            cfg_flags |= ConfigFlags::NAV_ENABLE_KEYBOARD | ConfigFlags::NAV_ENABLE_GAMEPAD;

            // Data search order: WD, executable path, user pref path, platform specific paths.
            let search_paths: Vec<&str> = vec![
                "",
                CStr::from_ptr(base_path).to_str()?,
                CStr::from_ptr(pref_path).to_str()?,
            ];

            #[cfg(not(target_os = "windows"))]
            search_paths.extend_from_slice(&PLATFORM_DATA_PATHS);
            pb::select_dat_file(
                &search_paths,
                &mut state.options_state,
                &mut state.pb_game_state,
            );

            // Second step: run updates that depend on .DAT file selection
            options::init_secondary(
                &mut state.options_state,
                &mut state.pb_game_state,
                &mut state.fullscrn_state,
            );

            // TODO: Implement sound, we're skipping for now to focus on PB:INIT();
            // match OPTIONS.lock() {
            //     Ok(options) => {
            //         sound::init(mix_opened, options.sound_channels, options.sounds, options.sound_volume);
            //     },
            //     Err(e) => {
            //         println!("Failed to lock options: {}", e);
            //     }
            // }

            if !pb::init(&mut state)? {
                let mut message = String::from(
                    "The .dat file is missing.\nMake sure that the game data is present in any of the following locations:",
                );
                for path in search_paths {
                    let str_push = if !path.is_empty() {
                        path
                    } else {
                        "working directory\n"
                    };
                    message += str_push;
                }
                println!("Could not load game data");
                pb::show_message_box(
                    SDL_MESSAGEBOX_ERROR,
                    "Could not load game data",
                    &message,
                    &state.main_state.main_window,
                )?;
                exit(1);
            }

            fullscrn::init(&mut state)?;

            pb::reset_table(&mut state.pb_game_state)?;
            pb::first_time_setup(&mut state.render_state, &mut state.pb_game_state);

            let fullscreen = env::args().any(|arg| arg == "-fullscreen");
            if fullscreen {
                *state.options_state.options.full_screen = true;
            }

            let res_val = state.fullscrn_state.resolution;
            if !*state.options_state.options.full_screen {
                let res_info = &state.fullscrn_state.resolution_array[res_val as usize];
                SDL_SetWindowSize(
                    window,
                    res_info.table_width as c_int,
                    res_info.table_height as c_int,
                );
            }
            SDL_ShowWindow(window);
            fullscrn::set_screen_mode(
                *state.options_state.options.full_screen,
                &mut state.fullscrn_state,
                &mut state.main_state.main_window,
            );

            let is_demo = env::args().any(|arg| arg == "-demo");
            if is_demo {
                // TODO LOWPRIO: Implement me
                pb::toggle_demo();
            } else {
                pb::replay_level(
                    false,
                    &mut state.main_state,
                    &mut state.options_state,
                    &mut state.pb_game_state,
                )?;
            }

            main_loop(&mut imgui_context, &mut state)?;

            options::uninit(&mut state.options_state);
            // TODO: Implement sound midi::music_shutdown();
            // TODO: Implement sound stuff
            //sound::close();
            pb::uninit(&mut state);

            if state.main_state.restart {
                println!("Restarting");
                restart(&mut state.main_state);
            }
        }
    }
}

fn build_glyph_ranges_from_translations() -> Vec<ImWchar> {
    let mut cps: Vec<u32> = Vec::new();
    for i in 0..(Msg::MAX as i32) {
        if let Some(msg) = Msg::from_i32(i)
            && let Ok(s) = translations::get_translation(msg)
        {
            for ch in s.chars() {
                cps.push(ch as u32);
            }
        }
    }

    // Regular ASCII
    for c in 0x20u32..=0x7Eu32 {
        cps.push(c);
    }

    cps.sort_unstable();
    cps.dedup();

    let mut ranges: Vec<ImWchar> = Vec::new();
    let mut cur_start: Option<u16> = Option::None;
    let mut cur_end: u16 = 0;
    for &cp in cps.iter() {
        if cp > 0xFFFF {
            continue;
        }
        let cp16 = cp as u16;
        if cur_start.is_none() {
            cur_start = Some(cp16);
            cur_end = cp16;
            continue;
        }
        if cp16 == cur_end || cp16 == cur_end + 1 {
            cur_end = cp16;
            continue;
        }
        ranges.push(cur_start.unwrap() as ImWchar);
        ranges.push(cur_end as ImWchar);
        cur_start = Some(cp16);
        cur_end = cp16;
    }
    if let Some(s) = cur_start {
        ranges.push(s as ImWchar);
        ranges.push(cur_end as ImWchar);
    }
    ranges.push(0); // Null terminator

    ranges
}

fn update_frame_rate(main_state: &mut MainState, options_state: &mut OptionsState) {
    let fps = options_state.options.frames_per_second.value;
    let ups = options_state.options.frames_per_second.value;
    main_state.update_to_frame_ratio = (ups as f64) / fps as f64;
    main_state.target_frametime = Duration((1000.0 / ups as f64) as i64);
}
