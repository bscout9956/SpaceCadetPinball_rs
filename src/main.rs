#![allow(unused)]
#![allow(arithmetic_overflow)]
extern crate core;

use crate::embedded_data::load_controller_db;
use crate::fullscrn::RESOLUTION_ARRAY;
use crate::options::{CONTROL_WAITING_FOR_INPUT, GameBindings, OptionsStruct};
use crate::translations::Msg;
use dear_imgui_rs::sys::ImGuiIO;
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
use std::cell::RefCell;
use std::env;
use std::error::Error;
use std::ffi::{CStr, CString, NulError, c_int};
use std::ops::{Index, Sub};
use std::path::PathBuf;
use std::process::exit;
use std::ptr::NonNull;
use std::str::FromStr;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32};
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
mod imgui_sdl;
pub mod message_code;
mod midi;
mod partman;
mod pb;
pub mod proj;
mod render;
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

struct WelfordState {
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
pub static SINGLE_STEP: AtomicBool = AtomicBool::new(false); // VERIFY default value
pub static LAUNCH_BALL_ENABLED: AtomicBool = AtomicBool::new(true);
pub static HIGH_SCORES_ENABLED: AtomicBool = AtomicBool::new(true);
pub static DEMO_ACTIVE: AtomicBool = AtomicBool::new(false);
pub static MAIN_MENU_HEIGHT: AtomicI32 = AtomicI32::new(0);

static RETURN_VALUE: AtomicI32 = AtomicI32::new(0);
static MOUSE_DOWN: AtomicBool = AtomicBool::new(false);
static LAST_MOUSE_X: AtomicI32 = AtomicI32::new(0);
static LAST_MOUSE_Y: AtomicI32 = AtomicI32::new(0);
static NO_TIME_LOSS: AtomicBool = AtomicBool::new(false);
static ACTIVATED: AtomicBool = AtomicBool::new(false);
static B_QUIT: AtomicBool = AtomicBool::new(false);
static HAS_FOCUS: AtomicBool = AtomicBool::new(true);
static DISP_GR_HISTORY: AtomicBool = AtomicBool::new(false);
static DISP_FRAME_RATE: AtomicBool = AtomicBool::new(false);
// TODO: CHECK DEFAULTS

static GFR_DISPLAY: Mutex<Vec<f32>> = Mutex::new(Vec::new());
static FPS_DETAILS: Mutex<String> = Mutex::new(String::new());
static PREV_SDL_ERROR: Mutex<String> = Mutex::new(String::new());
static RESTART: AtomicBool = AtomicBool::new(false);
static SHOW_ABOUT_DIALOG: AtomicBool = AtomicBool::new(false);
static SHOW_IMGUI_DEMO: AtomicBool = AtomicBool::new(false);
static SHOW_SPRITE_VIEWER: AtomicBool = AtomicBool::new(false);
static SHOW_EXIT_POPUP: AtomicBool = AtomicBool::new(false);

pub type DurationMs = f64;

static UPDATE_TO_FRAME_RATIO: Mutex<f64> = Mutex::new(0.0);
static TARGET_FRAMETIME: LazyLock<Mutex<Duration<1_000_000_000>>> =
    LazyLock::new(|| Mutex::new(Duration(0)));
static SPIN_THRESHOLD: LazyLock<Mutex<Duration<1_000_000_000>>> =
    LazyLock::new(|| Mutex::new(Duration(0)));
static SLEEP_STATE: LazyLock<Mutex<WelfordState>> =
    LazyLock::new(|| Mutex::new(WelfordState::new()));

static OPTIONS: &LazyLock<Mutex<OptionsStruct>> = &options::OPTIONS;
static PREV_SDL_ERROR_COUNT: AtomicU32 = AtomicU32::new(0);
static GFR_OFFSET: AtomicU32 = AtomicU32::new(0);
static CURSOR_IDLE_COUNTER: AtomicI32 = AtomicI32::new(0);

pub static MAIN_WINDOW: Mutex<Option<SdlWindowPtr>> = Mutex::new(Option::None);

pub fn get_main_menu_height() -> i32 {
    MAIN_MENU_HEIGHT.load(SeqCst)
}

pub static RENDERER: LazyLock<Mutex<Option<SdlRendererPtr>>> =
    LazyLock::new(|| Mutex::new(Option::None));

#[derive(Debug, Error)]
pub enum MainError {
    #[error("Renderer hasn't been assigned, is none")]
    NoneRendererError,

    #[error("Unable to lock Mutex for retrieving the SDL_Renderer on main")]
    MutexError(#[from] PoisonError<MutexGuard<'static, Option<SdlRendererPtr>>>),
    #[error("Failed to lock Mutex")]
    LockGeneric,
}

// TODO: Likewise
thread_local! {
    static IMGUI_IO: RefCell<Option<NonNull<ImGuiIO>>> = RefCell::new(Option::None);
}

pub fn set_imgui_io(io: *mut ImGuiIO) {
    IMGUI_IO.with(|cell| {
        let ptr = NonNull::new(io).expect("imgui io is null");
        *cell.borrow_mut() = Some(ptr);
    })
}

pub fn get_imgui_io() -> Option<NonNull<ImGuiIO>> {
    IMGUI_IO.with(|cell| *cell.borrow())
}

fn render_ui() {
    todo!()
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

fn main_loop(imgui_context: &mut Context) -> Result<(), MainLoopError> {
    B_QUIT.store(false, Relaxed);

    let mut update_count: usize = 0;
    let mut frame_counter: usize = 0;

    let frame_start = unsafe { SdlPerformanceClock::now() };
    let mut prev_time = frame_start;

    let _update_to_frame_counter = 0.0;
    let _sleep_remainder: Duration<1_000_000_000> = Duration(0);

    let target_frametime = TARGET_FRAMETIME
        .lock()
        .map_err(|e| MainLoopError::MutexLock)?;

    let _frame_duration = *target_frametime;

    loop {
        if DISP_FRAME_RATE.load(SeqCst) == true {
            let cur_time = unsafe { SdlPerformanceClock::now() };

            if (cur_time - prev_time) > Duration(1_000_000_000) {
                let elapsed_sec = (cur_time - prev_time).count() as f64 / 1_000_000_000f64;

                let title = format!(
                    "Updates/sec = {:02.2} Frames/sec = {:02.2} ",
                    update_count as f64 / elapsed_sec,
                    frame_counter as f64 / elapsed_sec
                );
                let c_str_title = CString::new(title.clone())?;

                let window_guard = MAIN_WINDOW.lock().map_err(|_| MainLoopError::MutexLock)?;
                if let Some(window) = window_guard.as_ref() {
                    unsafe {
                        SDL_SetWindowTitle(window.0, c_str_title.as_ptr());
                    };
                } else {
                    return Err(MainLoopError::NullWindow);
                }

                let mut fps_det = FPS_DETAILS.lock().map_err(|_| MainLoopError::MutexLock)?;
                *fps_det = String::from(&title);
                update_count = 0;
                frame_counter = update_count;
                prev_time = cur_time;
            }
        }

        if !process_window_messages(imgui_context)? || B_QUIT.load(SeqCst) == true {
            break;
        }

        if HAS_FOCUS.load(SeqCst) == true {
            if MOUSE_DOWN.load(SeqCst) > 0 {
                let mut x = 0;
                let mut y = 0;
                let mut w = 0;
                let mut h = 0;
                unsafe {
                    SDL_GetMouseState(&mut x, &mut y);

                    let window_guard = MAIN_WINDOW.lock().map_err(|_| MainLoopError::MutexLock)?;

                    if let Some(window) = window_guard.as_ref() {
                        SDL_GetWindowSize(window.0, &mut w, &mut h);
                    } else {
                        return Err(MainLoopError::NullWindow);
                    }
                }
                let dx = (LAST_MOUSE_X.load(SeqCst) - x) as f32 / w as f32;
                let dy = (y - LAST_MOUSE_Y.load(SeqCst)) as f32 / h as f32;
                pb::ball_set(dx, dy);

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
                    let window_guard = MAIN_WINDOW.lock().map_err(|_| MainLoopError::MutexLock)?;
                    if (x_mod != 0 || y_mod != 0) {
                        x = i32::abs(x as i32 - x_mod);
                        y = i32::abs(y as i32 - y_mod);
                        if let Some(window) = window_guard.as_ref() {
                            SDL_WarpMouseInWindow(window.0, x, y);
                        }
                    }
                }

                LAST_MOUSE_X.store(x, SeqCst);
                LAST_MOUSE_Y.store(y, SeqCst);
            }
        }
        if SINGLE_STEP.load(SeqCst) == false && NO_TIME_LOSS.load(SeqCst) == false {
            let dt = _frame_duration.count() as f32;
            pb::frame(dt);
            if DISP_GR_HISTORY.load(SeqCst) == true {
                // TODO: Continue from L360 in winmain.cpp
            }
        }

        NO_TIME_LOSS.store(false, SeqCst);

        let update_to_frame_ratio = UPDATE_TO_FRAME_RATIO
            .lock()
            .map_err(|_| MainLoopError::MutexLock)?;

        if _update_to_frame_counter >= *update_to_frame_ratio {
            let options = OPTIONS.lock().map_err(|_| MainLoopError::MutexLock)?;
            if *options.hide_cursor && CURSOR_IDLE_COUNTER.load(SeqCst) <= 0 {
                // TODO: ImGUiSetCursor l376
            }
            // TODO TODO TODO TODO, do all the todos above before continuing
        }
    }

    Ok(())
}

fn process_window_messages(imgui_context: &mut Context) -> Result<bool, MainLoopError> {
    static IDLE_WAIT: Mutex<i64> = Mutex::new(0);
    let event: *mut SDL_Event = unsafe { std::mem::zeroed() };

    let has_focus = HAS_FOCUS.load(SeqCst);
    if has_focus == true {
        let guard = IDLE_WAIT.lock().map_err(|_| MainLoopError::MutexLock)?;
        let frame_time_g = TARGET_FRAMETIME
            .lock()
            .map_err(|_| MainLoopError::MutexLock)?;

        *guard = (*frame_time_g).count();
        unsafe {
            while SDL_PollEvent(event) > 0 {
                if event_handler(event, imgui_context)? == 0 {
                    return Ok(false);
                }
            }
        }

        return Ok(true);
    }

    match IDLE_WAIT.lock().map_err(|_| MainLoopError::MutexLock) {
        Ok(mut idle_wait) => {
            let frame_time_g = TARGET_FRAMETIME
                .lock()
                .map_err(|_| MainLoopError::MutexLock)?;

            // Progressively wait longer when transitioning to idle
            *idle_wait = i64::min(*idle_wait + (*frame_time_g).0, 500);
            unsafe {
                if SDL_WaitEventTimeout(event, (*idle_wait) as c_int) > 0 {
                    *idle_wait = (*frame_time_g).count();
                    return event_handler(event);
                }
            }
            Ok(true)
        }
        Err(_) => Err(MainLoopError::MutexLock),
    }
}

unsafe fn event_handler(
    event: *mut SDL_Event,
    imgui_context: &mut Context,
) -> Result<i32, MainLoopError> {
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

    let waiting_input = CONTROL_WAITING_FOR_INPUT
        .lock()
        .map_err(|_| MainLoopError::MutexLock)?;

    if (*waiting_input).is_none() || !input_down {
        imgui_sdl::impl_sdl2_process_event(imgui_context, event);
    }

    let mouse_event: bool;

    unsafe {
        if (*event).type_ == SDL_EventType::SDL_MOUSEMOTION as u32
            || (*event).type_ == SDL_EventType::SDL_MOUSEBUTTONDOWN as u32
            || (*event).type_ == SDL_EventType::SDL_MOUSEBUTTONUP as u32
            || (*event).type_ == SDL_EventType::SDL_MOUSEWHEEL as u32
        {
            CURSOR_IDLE_COUNTER.store(1000, SeqCst);
            mouse_event = true;
        } else {
            mouse_event = false;
        }
    }

    let io = imgui_context.io_mut();
    let waiting_for_input_grd = CONTROL_WAITING_FOR_INPUT
        .lock()
        .map_err(|_| MainLoopError::MutexLock)?;

    if io.want_capture_mouse() && waiting_for_input_grd.is_none() {
        if MOUSE_DOWN.load(SeqCst) == true {
            MOUSE_DOWN.store(false, SeqCst);
            let main_window_grd = MAIN_WINDOW.lock().map_err(|_| MainLoopError::MutexLock)?;
            if let Some(window) = main_window_grd.as_ref() {
                unsafe {
                    SDL_SetWindowGrab(window.0, SDL_FALSE);
                }
            }
        }

        if mouse_event {
            return Ok(1);
        }
    }

    if io.want_capture_keyboard() && waiting_for_input_grd.is_none() {
        unsafe {
            if (*event).type_ == SDL_KEYUP as u32
                || (*event).type_ == SDL_KEYDOWN as u32
                || (*event).type_ == SDL_CONTROLLERBUTTONDOWN as u32
                || (*event).type_ == SDL_CONTROLLERBUTTONUP as u32
            {
                return Ok(1);
            }
        }
    }

    if (*event).type_ == SDL_QUIT as u32 {
        end_pause();
        B_QUIT.store(true, SeqCst);
        fullscrn::shutdown();
        return_value = 0;
        return Ok(0);
    }
    if (*event).type_ == SDL_KEYUP as u32 {
        pb::input_up()
    }

    Ok(1)
}

fn main() -> Result<(), Box<dyn Error>> {
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
            );
            println!("OOPS!! No init, closing");
            exit(1);
        }
    }

    let quick_flag = args.iter().any(|arg| arg.contains("-quick"));
    pb::QUICK_FLAG.store(quick_flag, Relaxed);

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

        match MAIN_WINDOW.try_lock() {
            Ok(mut main_window_grd) => {
                *main_window_grd = main_window;
            }
            Err(std::sync::TryLockError::Poisoned(_)) => {
                println!("Poisoned lock because a thread panicked.");
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                println!("Another thread is locking MAIN_WINDOW");
            }
        }

        if MAIN_WINDOW.lock()?.is_none() {
            pb::show_message_box_cstr_message(
                SDL_MESSAGEBOX_ERROR,
                "Could not create window",
                SDL_GetError(),
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
                let mut static_renderer = RENDERER.lock()?;
                *static_renderer = Some(SdlRendererPtr(renderer));
                println!("Renderer successfully created and assigned.");
                break;
            }
        }

        if RENDERER.lock()?.is_none() {
            pb::show_message_box_cstr_message(
                SDL_MESSAGEBOX_ERROR,
                "Could not create renderer",
                SDL_GetError(),
            );
            println!("Could not create renderer, is null");
            exit(1);
        }

        match RENDERER.lock() {
            Ok(guard) => {
                let mut renderer_info: SDL_RendererInfo = std::mem::zeroed();

                if let Some(renderer_ptr) = guard.as_ref() {
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
            }
            Err(e) => {
                println!("Error locking renderer: {}", e);
            }
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
            RESTART.store(false, Relaxed);

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
            options::init_primary();
            if reset_all_options {
                reset_all_options = false;
                options::reset_all_options();
            }

            match OPTIONS.lock() {
                Ok(options) => {
                    let font_file_name = &options.font_file_name.value;

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
                }
                Err(err) => {
                    println!("Failed to lock options: {}", err);
                }
            }

            match RENDERER.lock() {
                Ok(renderer_ptr) => {
                    if let Some(renderer) = renderer_ptr.as_ref() {
                        println!("Initializing IMGUI_SDL");
                        imgui_sdl::initialize(imgui_context, renderer.0, 0, 0);

                        imgui_sdl::init_for_sdl_renderer(imgui_context, window, renderer.0);
                    } else {
                        panic!("No renderer found to initialize IMGUI!");
                    }
                }
                Err(e) => {
                    println!("Failed to lock renderer: {}", e)
                }
            }

            cfg_flags |= ConfigFlags::NAV_ENABLE_KEYBOARD | ConfigFlags::NAV_ENABLE_GAMEPAD;

            // Data search order: WD, executable path, user pref path, platform specific paths.
            let mut search_paths: Vec<&str> = Vec::new();
            search_paths.push("");
            search_paths.push(CStr::from_ptr(base_path).to_str()?);
            search_paths.push(CStr::from_ptr(pref_path).to_str()?);

            #[cfg(not(target_os = "windows"))]
            search_paths.extend_from_slice(&PLATFORM_DATA_PATHS);
            pb::select_dat_file(&search_paths);

            // Second step: run updates that depend on .DAT file selection
            options::init_secondary();

            // TODO: Implement sound, we're skipping for now to focus on PB:INIT();
            // match OPTIONS.lock() {
            //     Ok(options) => {
            //         sound::init(mix_opened, options.sound_channels, options.sounds, options.sound_volume);
            //     },
            //     Err(e) => {
            //         println!("Failed to lock options: {}", e);
            //     }
            // }

            if !pb::init()? {
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
                pb::show_message_box(SDL_MESSAGEBOX_ERROR, "Could not load game data", &message);
                exit(1);
            }

            fullscrn::init();

            pb::reset_table();
            pb::first_time_setup();

            let fullscreen = env::args().any(|arg| arg == "-fullscreen");
            if fullscreen {
                let mut options = OPTIONS.lock()?;
                *options.full_screen = true;
            }

            {
                let options = OPTIONS.lock()?;
                if *options.full_screen == false {
                    let resolution_array = RESOLUTION_ARRAY.lock()?;
                    let res_info = &resolution_array[fullscrn::get_resolution() as usize];
                    SDL_SetWindowSize(
                        window,
                        res_info.table_width as c_int,
                        res_info.table_height as c_int,
                    );
                }
                SDL_ShowWindow(window);
                fullscrn::set_screen_mode(*options.full_screen);
            }

            let is_demo = env::args().any(|arg| arg == "-demo");
            if is_demo {
                // TODO LOWPRIO: Implement me
                pb::toggle_demo();
            } else {
                pb::replay_level(false);
            }

            main_loop();

            options::uninit();
            midi::music_shutdown();
            // TODO: Implement sound stuff
            //sound::close();
            pb::uninit();

            let do_restart = RESTART.load(Relaxed);
            if do_restart {
                ()
            }
        }
    }
}
