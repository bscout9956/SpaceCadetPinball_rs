#![allow(unused)]
#![allow(arithmetic_overflow)]
extern crate core;

use crate::options::{GameBindings, OptionsStruct};
use dear_imgui_rs::sys::ImGuiIO;
use lazy_static::lazy_static;
use sdl2::sys::{
    SDL_GetPerformanceCounter, SDL_GetPerformanceFrequency, SDL_GetTicks, SDL_Renderer,
};
use sdl2::{
    sys::{
        SDL_CreateWindow, SDL_INIT_AUDIO, SDL_INIT_EVENTS, SDL_INIT_GAMECONTROLLER,
        SDL_INIT_JOYSTICK, SDL_INIT_TIMER, SDL_INIT_VIDEO, SDL_Init, SDL_SetMainReady, SDL_Window,
        SDL_WindowFlags::{SDL_WINDOW_HIDDEN, SDL_WINDOW_RESIZABLE},
    },
    video::WindowPos,
};
use std::cell::RefCell;
use std::process::exit;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32};
use std::sync::{LazyLock, Mutex};
use std::time::Duration as StdDuration;
use std::time::Instant;

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

mod midi;
mod pb;
mod render;
mod utils;

pub fn restart_func() {
    todo!();
}

pub struct Duration<const DEN: u64>(i64);
pub struct TimePoint<const DEN: u64>(Duration<DEN>);

pub trait Clock {
    type Duration;
    type TimePoint;
    const IS_STEADY: bool;
    unsafe fn now() -> Self::TimePoint;
}

pub struct SdlPerformanceClock;
impl Clock for SdlPerformanceClock {
    type Duration = Duration<1_000_000_000>;
    type TimePoint = TimePoint<1_000_000_000>;
    const IS_STEADY: bool = true;
    unsafe fn now() -> Self::TimePoint {
        let freq = SDL_GetPerformanceFrequency();
        let ctr = SDL_GetPerformanceCounter();
        let whole = (ctr / freq) * 1_000_000_000;
        let part = (ctr % freq) * 1_000_000_000 / freq;
        TimePoint(Duration((whole + part) as i64))
    }
}

pub struct SdlTickClock;
impl Clock for SdlTickClock {
    type Duration = Duration<1_000>;
    type TimePoint = TimePoint<1_000>;
    const IS_STEADY: bool = true;
    unsafe fn now() -> Self::TimePoint {
        let ticks = SDL_GetTicks();
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
        return f64::sqrt(self.m2 / (self.count - 1) as f64);
    }
}

pub const VERSION: &str = "1.0 DEV";
pub static SINGLE_STEP: AtomicBool = AtomicBool::new(false); // VERIFY default value
pub static LAUNCH_BALL_ENABLED: AtomicBool = AtomicBool::new(true);
pub static HIGH_SCORES_ENABLED: AtomicBool = AtomicBool::new(true);
pub static DEMO_ACTIVE: AtomicBool = AtomicBool::new(false);
pub static MAIN_MENU_HEIGHT: AtomicI32 = AtomicI32::new(0);

static RETURN_VALUE: AtomicI32 = AtomicI32::new(0);
// VERIFY: Switch to bool?
static MOUSE_DOWN: AtomicI32 = AtomicI32::new(0);
static LAST_MOUSE_X: AtomicI32 = AtomicI32::new(0);
static LAST_MOUSE_Y: AtomicI32 = AtomicI32::new(0);
static NO_TIME_LOSS: AtomicBool = AtomicBool::new(false);
static ACTIVATED: AtomicBool = AtomicBool::new(false);
static B_QUIT: AtomicBool = AtomicBool::new(false);
static HAS_FOCUS: AtomicBool = AtomicBool::new(true);
static DISP_GR_HISTORY: AtomicBool = AtomicBool::new(false);
static DISP_FRAME_RATE: AtomicBool = AtomicBool::new(false);
// TODO: CHECK DEFAULTS
lazy_static! {
    static ref GFR_DISPLAY: Mutex<Vec<f32>> = Mutex::new(Vec::new());
    static ref FPS_DETAILS: Mutex<String> = Mutex::new(String::new());
    static ref PREV_SDL_ERROR: Mutex<String> = Mutex::new(String::new());
}
static RESTART: AtomicBool = AtomicBool::new(false);
static SHOW_ABOUT_DIALOG: AtomicBool = AtomicBool::new(false);
static SHOW_IMGUI_DEMO: AtomicBool = AtomicBool::new(false);
static SHOW_SPRITE_VIEWER: AtomicBool = AtomicBool::new(false);
static SHOW_EXIT_POPUP: AtomicBool = AtomicBool::new(false);

pub type DurationMs = f64;

lazy_static! {
    static ref UPDATE_TO_FRAME_RATIO: Mutex<f64> = Mutex::new(0.0);
    static ref TARGET_FRAMETIME: Mutex<DurationMs> = Mutex::new(DurationMs::default());
    static ref SPIN_THRESHOLD: Mutex<DurationMs> = Mutex::new(DurationMs::default());
    static ref SLEEP_STATE: Mutex<WelfordState> = Mutex::new(WelfordState::new());
}

static OPTIONS: &LazyLock<Mutex<OptionsStruct>> = &options::OPTIONS;
static PREV_SDL_ERROR_COUNT: AtomicU32 = AtomicU32::new(0);
static GFR_OFFSET: AtomicU32 = AtomicU32::new(0);
static CURSOR_IDLE_COUNTER: AtomicI32 = AtomicI32::new(0);

// TODO: If I realize that I'll need to use this on threads, use OnceLock/LazyLock w/ Mutex<Option>
thread_local! {
    static MAIN_WINDOW: RefCell<Option<NonNull<SDL_Window>>> = RefCell::new(None);
}

pub fn set_main_window(window: *mut SDL_Window) {
    MAIN_WINDOW.with(|cell| {
        let ptr = NonNull::new(window).expect("window is null");
        *cell.borrow_mut() = Some(ptr);
    })
}

pub fn get_main_window() -> Option<NonNull<SDL_Window>> {
    MAIN_WINDOW.with(|cell| *cell.borrow())
}

// TODO: If I realize that I'll need to use this on threads, use OnceLock or Mutex<Option>
thread_local! {
    static RENDERER: RefCell<Option<NonNull<SDL_Renderer>>> = RefCell::new(None);
}

pub fn set_renderer(renderer: *mut SDL_Renderer) {
    RENDERER.with(|cell| {
        let ptr = NonNull::new(renderer).expect("renderer is null");
        *cell.borrow_mut() = Some(ptr);
    })
}

pub fn get_renderer() -> Option<NonNull<SDL_Renderer>> {
    RENDERER.with(|cell| *cell.borrow())
}

// TODO: Likewise
thread_local! {
    static IMGUI_IO: RefCell<Option<NonNull<ImGuiIO>>> = RefCell::new(None);
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

fn render_ui() {}

fn render_frame_time_dialog() {}

fn hybrid_sleep(seconds: DurationMs) {}

fn main_loop() {}

// bool defaults to false
fn imgui_menu_item_w_shortcut(binding: GameBindings, selected: Option<bool>) {}

fn main() {
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
            println!("OOPS!! No init, closing");
            exit(1);
        }
    }
    unsafe {
        println!("Creating window");
        let window = SDL_CreateWindow(
            c"PinBall Space Cadet (0.0.0)".as_ptr(),
            0,
            0,
            800,
            556,
            SDL_WINDOW_HIDDEN as u32 | SDL_WINDOW_RESIZABLE as u32,
        );
    }
}
