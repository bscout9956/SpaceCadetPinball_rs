#![allow(unused)]
#![allow(arithmetic_overflow)]
extern crate core;

use crate::embedded_data::load_controller_db;
use crate::options::GameBindings;
use crate::translations::Msg;
use dear_imgui_rs::sys::{
    ImGuiCol_MenuBarBg, ImGuiIO, ImGuiMouseCursor_None, ImVec4, igNewFrame, igPushStyleColor_Vec4,
    igSetMouseCursor,
};
use dear_imgui_rs::{ConfigFlags, Context, FontConfig};
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
use state::fullscrn_state::FullscrnState;
use state::main_state::MainState;
use state::options_state::OptionsState;
use state::pinball_state::PinballState;
use std::cell::RefCell;
use std::env;
use std::error::Error;
use std::ffi::{CStr, CString, NulError, c_int};
use std::mem::MaybeUninit;
use std::ops::{Index, Sub};
use std::path::PathBuf;
use std::process::exit;
use std::ptr::NonNull;
use std::str::FromStr;
use std::sync::atomic::AtomicU32;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};
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

pub struct SdlWindowPtr(pub *mut SDL_Window);
unsafe impl Sync for SdlWindowPtr {}
unsafe impl Send for SdlWindowPtr {}
pub struct SdlRendererPtr(pub *mut SDL_Renderer);
unsafe impl Send for SdlRendererPtr {}
unsafe impl Sync for SdlRendererPtr {}

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

fn render_frame_time_dialog() {
    todo!()
}

fn hybrid_sleep(seconds: DurationMs) {
    todo!()
}

// TODO: Implement?
// bool defaults to false
fn imgui_menu_item_w_shortcut(binding: GameBindings, selected: Option<bool>) {}

#[derive(Error, Debug)]
pub enum MainLoopError {
    #[error("Failed to lock Mutex")]
    MutexLock,
    #[error(transparent)]
    NulError(#[from] NulError),
    #[error("There is no MainWindow to attach to...")]
    NullWindow,
}

fn main_loop(
    imgui_context: &mut Context,
    pb_state: &mut PinballState,
) -> Result<(), MainLoopError> {
    pb_state.main_state.b_quit = false;

    let mut update_count: usize = 0;
    let mut frame_counter: usize = 0;

    let frame_start = unsafe { SdlPerformanceClock::now() };
    let mut prev_time = frame_start;

    let _update_to_frame_counter = 0.0;
    let _sleep_remainder: Duration<1_000_000_000> = Duration(0);

    let _frame_duration = pb_state.main_state.target_frametime;

    loop {
        if (&mut pb_state.main_state).disp_frame_rate == true {
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

        if !process_window_messages(
            imgui_context,
            &mut pb_state.main_state,
            &mut pb_state.options_state,
            &mut pb_state.fullscrn_state,
        )? || (&mut pb_state.main_state).b_quit == false
        {
            break;
        }

        if (&mut pb_state.main_state).has_focus {
            if (&mut pb_state.main_state).mouse_down {
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
                let dx = ((&mut pb_state.main_state).last_mouse_x - x) as f32 / w as f32;
                let dy = (y - (&mut pb_state.main_state).last_mouse_y) as f32 / h as f32;
                pb::ball_set(dx, dy, &mut pb_state.pb_game_state);

                // Original creates continuous mouse movement with mouse capture.
                // Alternative solution: mouse warp at window edges.
                let mut x_mod: i32 = 0;
                let mut y_mod: i32 = 0;

                if (x as i32 == 0 || x as i32 >= (w as i32 - 1)) {
                    x_mod = w as i32 - 2;
                }
                if (y as i32 == 0 || y as i32 >= (h as i32 - 1)) {
                    y_mod = h as i32 - 2;
                }

                unsafe {
                    if (x_mod != 0 || y_mod != 0) {
                        x = i32::abs(x as i32 - x_mod);
                        y = i32::abs(y as i32 - y_mod);
                        if let Some(window) = pb_state.main_state.main_window.as_ref() {
                            SDL_WarpMouseInWindow(window.0, x, y);
                        }
                    }
                }

                (&mut pb_state.main_state).update_mouse_xy(x, y);
            }
        }
        if (&mut pb_state.main_state).single_step == false
            && (&mut pb_state.main_state).no_time_loss == false
        {
            let dt = _frame_duration.count() as f32;
            pb::frame(dt, &mut pb_state.pb_game_state);
            if (&mut pb_state.main_state).disp_gr_history == true {
                // TODO: Continue from L360 in winmain.cpp
            }
        }

        (&mut pb_state.main_state).no_time_loss = false;

        if _update_to_frame_counter >= (&mut pb_state.main_state).update_to_frame_ratio {
            if *pb_state.options_state.options.hide_cursor
                && (&mut pb_state.main_state).cursor_idle_counter <= 0
            {
                // TODO: ImGUiSetCursor l376
                unsafe { igSetMouseCursor(ImGuiMouseCursor_None) };
                // imgui_sdl::impl_sdl2_new_frame(); TODO
                // imgui_sdl::render_new_frame(); TODO
                unsafe {
                    igNewFrame();
                    render_ui();

                    SDL_RenderClear((&mut pb_state.main_state).renderer.as_ref().unwrap().0) // TODO: If let Some here
                };
            }
            // TODO TODO TODO TODO, do all the todos above before continuing
        }
    }

    Ok(())
}

unsafe fn render_ui() {
    let vec4 = ImVec4::new(0.0, 0.0, 0.0, 1.0);
    unsafe {
        igPushStyleColor_Vec4(ImGuiCol_MenuBarBg, vec4);
    }
}

fn process_window_messages(
    imgui_context: &mut Context,
    main_state: &mut MainState,
    options_state: &mut OptionsState,
    fullscrn_state: &mut FullscrnState,
) -> Result<bool, MainLoopError> {
    let mut idle_wait = 0i64;
    let mut event = MaybeUninit::<SDL_Event>::uninit();

    if main_state.has_focus {
        idle_wait = main_state.target_frametime.count();
        unsafe {
            while SDL_PollEvent(event.as_mut_ptr()) > 0 {
                if event_handler(
                    event.as_mut_ptr(),
                    imgui_context,
                    main_state,
                    options_state,
                    fullscrn_state,
                )? == false
                {
                    return Ok(false);
                }
            }
        }

        return Ok(true);
    }

    // Progressively wait longer when transitioning to idle
    idle_wait = i64::min(idle_wait + main_state.target_frametime.0, 500);
    unsafe {
        if SDL_WaitEventTimeout(event.as_mut_ptr(), idle_wait as c_int) > 0 {
            idle_wait = main_state.target_frametime.count();
            return event_handler(
                event.as_mut_ptr(),
                imgui_context,
                main_state,
                options_state,
                fullscrn_state,
            );
        }
    }
    Ok(true)
}

unsafe fn event_handler(
    event: *mut SDL_Event,
    imgui_context: &mut Context,
    main_state: &mut MainState,
    options_state: &mut OptionsState,
    fullscrn_state: &mut FullscrnState,
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

    if options_state.control_waiting_for_input.is_none() || !input_down {
        imgui_sdl::impl_sdl2_process_event(imgui_context, event);
    }

    let mouse_event: bool;

    unsafe {
        if (*event).type_ == SDL_EventType::SDL_MOUSEMOTION as u32
            || (*event).type_ == SDL_EventType::SDL_MOUSEBUTTONDOWN as u32
            || (*event).type_ == SDL_EventType::SDL_MOUSEBUTTONUP as u32
            || (*event).type_ == SDL_EventType::SDL_MOUSEWHEEL as u32
        {
            main_state.cursor_idle_counter = 1000;
            mouse_event = true;
        } else {
            mouse_event = false;
        }
    }

    let io = imgui_context.io_mut();

    if io.want_capture_mouse() && options_state.control_waiting_for_input.is_none() {
        if main_state.mouse_down == true {
            main_state.mouse_down = false;
            if let Some(window) = main_state.main_window.as_ref() {
                unsafe {
                    SDL_SetWindowGrab(window.0, SDL_FALSE);
                }
            }
        }

        if mouse_event {
            return Ok(true);
        }
    }

    if io.want_capture_keyboard() && options_state.control_waiting_for_input.is_none() {
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
            end_pause(main_state);

            main_state.b_quit = true;
            fullscrn::shutdown(fullscrn_state, &mut main_state.main_window);
            main_state.return_value = 0;
            return Ok(false);
        }
        if (*event).type_ == SDL_KEYUP as u32 {
            pb::input_up()
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
        let rc_cstr = CStr::from_bytes_with_nul_unchecked(rc_string.as_bytes());

        let window = SDL_CreateWindow(
            rc_cstr.as_ptr(),
            0,
            0,
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
            (&mut state.main_state).restart = false;

            // ImGUi Init
            let mut imgui_context = Context::create();
            let io = imgui_context.io_mut();
            let mut cfg_flags = io.config_flags();

            let font_cfg = FontConfig::new().oversample_h(2).oversample_h(4);

            let pref_path_string =
                CStr::from_ptr(pref_path).to_string_lossy().into_owned() + "imgui_pb.ini";
            let mut ini_path = PathBuf::from(pref_path_string);

            imgui_context.set_ini_filename(Some(ini_path));

            // First option initialization step: just load settings from .ini. Needs ImGui context.
            options::init_primary(&mut state.options_state);
            if reset_all_options {
                reset_all_options = false;
                options::reset_all_options(&mut state.options_state);
            }

            let font_file_name = &(&mut state.options_state).options.font_file_name.value;

            if !font_file_name.is_empty() {
                let mut fonts = imgui_context.fonts();
                let ranges = fonts.get_glyph_ranges_default().to_vec();

                let custom_font = fonts.add_font_from_file_ttf(
                    font_file_name,
                    13.0,
                    Some(&font_cfg),
                    Some(&ranges),
                );

                if custom_font.is_none() {
                    println!("Could not load font {}", font_file_name);
                    imgui_context.fonts().add_font_default(Some(&font_cfg));
                }
            } else {
                imgui_context.fonts().add_font_default(Some(&font_cfg));
            }

            imgui_context.fonts().build();

            if let Some(renderer) = (&mut state.main_state).renderer.as_ref() {
                println!("Initializing IMGUI_SDL");
                imgui_sdl::initialize(&mut imgui_context, renderer.0, 0, 0);
                imgui_sdl::init_for_sdl_renderer(&mut imgui_context, window, renderer.0);
            } else {
                panic!("No renderer found to initialize IMGUI!");
            }

            cfg_flags |= ConfigFlags::NAV_ENABLE_KEYBOARD | ConfigFlags::NAV_ENABLE_GAMEPAD;

            // Data search order: WD, executable path, user pref path, platform specific paths.
            let mut search_paths: Vec<&str> = Vec::new();
            search_paths.push("");
            search_paths.push(CStr::from_ptr(base_path).to_str()?);
            search_paths.push(CStr::from_ptr(pref_path).to_str()?);

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
                );
                exit(1);
            }

            fullscrn::init(&mut state);

            pb::reset_table(&mut state.pb_game_state);
            pb::first_time_setup(&mut state.render_state);

            let fullscreen = env::args().any(|arg| arg == "-fullscreen");
            if fullscreen {
                *(&mut state.options_state).options.full_screen = true;
            }

            let res_val = state.fullscrn_state.resolution;
            if *(&mut state.options_state).options.full_screen == false {
                let res_info = &(&mut state.fullscrn_state).resolution_array[res_val as usize];
                SDL_SetWindowSize(
                    window,
                    res_info.table_width as c_int,
                    res_info.table_height as c_int,
                );
            }
            SDL_ShowWindow(window);
            fullscrn::set_screen_mode(
                *(&mut state.options_state).options.full_screen,
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

            main_loop(&mut imgui_context, &mut state);

            options::uninit(&mut state.options_state);
            // TODO: Implement sound midi::music_shutdown();
            // TODO: Implement sound stuff
            //sound::close();
            pb::uninit(&mut state.pb_game_state);

            if (&mut state.main_state).restart {
                ()
            }
        }
    }
}
