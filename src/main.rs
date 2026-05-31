#![allow(unused)]
#![allow(arithmetic_overflow)]
extern crate core;

use crate::embedded_data::load_controller_db;
use crate::options::{GameBindings, OptionsStruct};
use crate::translations::Msg;
use dear_imgui_rs::sys::ImGuiIO;
use dear_imgui_rs::{Context, FontConfig};
use lazy_static::lazy_static;
use sdl2::controller::AddMappingError;
use sdl2::libc::strstr;
use sdl2::mixer::get_linked_version;
use sdl2::rwops::RWops;
use sdl2::sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_ERROR;
use sdl2::sys::SDL_RendererFlags::{SDL_RENDERER_ACCELERATED, SDL_RENDERER_SOFTWARE};
use sdl2::sys::mixer::{
    MIX_DEFAULT_FORMAT, MIX_DEFAULT_FREQUENCY, MIX_InitFlags_MIX_INIT_MID, MIX_MAJOR_VERSION,
    MIX_MINOR_VERSION, MIX_PATCHLEVEL, Mix_Init, Mix_OpenAudio,
};
use sdl2::sys::{
    SDL_ClearError, SDL_CreateRenderer, SDL_GameControllerAddMappingsFromRW, SDL_GetBasePath,
    SDL_GetError, SDL_GetPerformanceCounter, SDL_GetPerformanceFrequency, SDL_GetPrefPath,
    SDL_GetRendererInfo, SDL_GetTicks, SDL_GetVersion, SDL_HINT_RENDER_SCALE_QUALITY,
    SDL_MAJOR_VERSION, SDL_MINOR_VERSION, SDL_PATCHLEVEL, SDL_Renderer, SDL_RendererInfo,
    SDL_SetHint, SDL_SetRenderDrawColor, SDL_ShowWindow,
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
use std::ffi::{CStr, CString, c_int};
use std::path::PathBuf;
use std::process::exit;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32};
use std::sync::{LazyLock, LockResult, Mutex};
use std::time::Duration as StdDuration;
use std::time::Instant;
use std::{env, ptr};

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

mod embedded_data;
mod imgui_sdl;
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

static RENDERER: LazyLock<Mutex<Option<SDL_Renderer>>> = LazyLock::new(|| Mutex::new(None));

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
    println!("Game version: {}", VERSION);
    let args: Vec<String> = std::env::args().collect();
    println!("Command line: {:?}", args);
    print!(
        "Compiled with: SDL {}.{}.{}",
        SDL_MAJOR_VERSION, SDL_MINOR_VERSION, SDL_PATCHLEVEL
    );
    print!(
        " SDL_mixer {}.{}.{};",
        MIX_MAJOR_VERSION, MIX_MINOR_VERSION, MIX_PATCHLEVEL
    );
    println!(" ImGui {}", "TODO");

    let sdl_context = sdl2::init().unwrap();
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
            // TODO: ShowMessageBox
            // pb::ShowMessageBox(SDL_MESSAGEBOX_ERROR, "Could not initialize SDL2", SDL_GetError());
            println!("OOPS!! No init, closing");
            exit(1);
        }
    }

    let quick_flag = args.iter().any(|arg| arg.contains("-quick"));
    pb::QUICK_FLAG.store(quick_flag, Relaxed);

    unsafe {
        println!("Creating window");
        let window = SDL_CreateWindow(
            // TODO: Implement
            // pb::get_rc_string(Msg::STRING139),
            c"PinBall Space Cadet (0.0.0)".as_ptr(),
            0,
            0,
            800,
            556,
            SDL_WINDOW_HIDDEN as u32 | SDL_WINDOW_RESIZABLE as u32,
        );
        let mut main_window = get_main_window();
        set_main_window(window);
        main_window = get_main_window();
        if !main_window.is_some() {
            // TODO: Implement ShowMSGBOX
            //  pb::ShowMessageBox(SDL_MESSAGEBOX_ERROR, "Could not create window", SDL_GetError());
            println!("Could not create window");
            exit(1);
        }

        let sw_offset_flag = args.iter().any(|arg| arg.contains("-sw"));
        let mut renderer: *mut SDL_Renderer = std::mem::zeroed();
        for i in sw_offset_flag as i32..2 {
            println!("Offset {}", i);
            let flags = if i == 0 {
                print!("Using HW accel");
                SDL_RENDERER_ACCELERATED
            } else {
                println!("Using software");
                SDL_RENDERER_SOFTWARE
            };
            renderer = SDL_CreateRenderer(window, -1, flags as u32);
            let mut static_render = RENDERER.lock().unwrap();
            if !renderer.is_null() {
                *static_render = Some(*renderer);
                println!("Renderer successfully created and assigned.");
            }

            if !renderer.is_null() {
                println!("Renderer is not null");
                break;
            }
        }

        if renderer.is_null() {
            // TODO: Implement me
            //pb::ShowMessageBox(
            //    SDL_MESSAGEBOX_ERROR,
            //    "Could not create renderer",
            //    SDL_GetError(),
            //);
            println!("Could not create renderer, is null");
            exit(1);
        }
        let mut renderer_info: SDL_RendererInfo = std::mem::zeroed();
        let result = SDL_GetRendererInfo(renderer, &mut renderer_info);
        if result != 0 {
            println!("Error getting renderer information");
        } else {
            println!(
                "Using SDL Renderer: {}",
                CStr::from_ptr(renderer_info.name).to_str().unwrap()
            );
        }
        SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
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
                    CStr::from_ptr(SDL_GetError()).to_str().unwrap()
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
                    CStr::from_ptr(SDL_GetError()).to_str().unwrap()
                );
                SDL_ClearError();
            } else {
                println!("Mix opened!");
                mix_opened = true;
            }
        }

        // Load SDL Game Controller definitions from DB
        // This is more Rust idiomatic because all solutions I've tried were a complete nightmare...
        // Including trying to mimick the original one
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
            let font_cfg = FontConfig::new().oversample_h(2).oversample_h(4);

            let pref_path_string = CStr::from_ptr(pref_path).to_str().unwrap().to_owned();
            let mut ini_path = PathBuf::from(pref_path_string);
            ini_path.push("imgui_pb.ini");

            imgui_context.set_ini_filename(Some(ini_path));

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

            imgui_sdl::initialize(&mut imgui_context, renderer, 0, 0);
            
            SDL_ShowWindow(window);

            let do_restart = RESTART.load(Relaxed);
            if do_restart {
                break;
            }
        }
    }
}
