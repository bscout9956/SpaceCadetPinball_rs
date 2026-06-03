#![allow(unused)]
#![allow(arithmetic_overflow)]
extern crate core;

use crate::embedded_data::load_controller_db;
use crate::options::{GameBindings, OptionsStruct};
use dear_imgui_rs::sys::ImGuiIO;
use dear_imgui_rs::{ConfigFlags, Context, FontConfig};
use sdl2::sys::SDL_RendererFlags::{SDL_RENDERER_ACCELERATED, SDL_RENDERER_SOFTWARE};
use sdl2::sys::SDL_WindowFlags::{SDL_WINDOW_HIDDEN, SDL_WINDOW_RESIZABLE};
use sdl2::sys::mixer::{
    MIX_DEFAULT_FORMAT, MIX_DEFAULT_FREQUENCY, MIX_InitFlags_MIX_INIT_MID, MIX_MAJOR_VERSION,
    MIX_MINOR_VERSION, MIX_PATCHLEVEL, Mix_Init, Mix_OpenAudio,
};
use sdl2::sys::*;
use std::cell::RefCell;
use std::env;
use std::error::Error;
use std::ffi::{CStr, c_int};
use std::ops::Index;
use std::path::PathBuf;
use std::process::exit;
use std::ptr::{NonNull, addr_of_mut};
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

mod embedded_data;
mod errors;
mod imgui_sdl;
mod midi;
mod partman;
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
        let freq = unsafe { SDL_GetPerformanceFrequency() };
        let ctr = unsafe { SDL_GetPerformanceCounter() };
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
        f64::sqrt(self.m2 / (self.count - 1) as f64)
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
static TARGET_FRAMETIME: LazyLock<Mutex<DurationMs>> =
    LazyLock::new(|| Mutex::new(DurationMs::default()));
static SPIN_THRESHOLD: LazyLock<Mutex<DurationMs>> =
    LazyLock::new(|| Mutex::new(DurationMs::default()));
static SLEEP_STATE: LazyLock<Mutex<WelfordState>> =
    LazyLock::new(|| Mutex::new(WelfordState::new()));

static OPTIONS: &LazyLock<Mutex<OptionsStruct>> = &options::OPTIONS;
static PREV_SDL_ERROR_COUNT: AtomicU32 = AtomicU32::new(0);
static GFR_OFFSET: AtomicU32 = AtomicU32::new(0);
static CURSOR_IDLE_COUNTER: AtomicI32 = AtomicI32::new(0);

// TODO: If I realize that I'll need to use this on threads, use OnceLock/LazyLock w/ Mutex<Option>
thread_local! {
    static MAIN_WINDOW: RefCell<Option<NonNull<SDL_Window>>> = RefCell::new(Option::None);
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

pub fn get_main_menu_height() -> i32 {
    MAIN_MENU_HEIGHT.load(SeqCst)
}

static RENDERER: LazyLock<Mutex<Option<SDL_Renderer>>> = LazyLock::new(|| Mutex::new(Option::None));

#[derive(Debug, Error)]
pub enum MainError {
    #[error("Renderer hasn't been assigned, is none")]
    NoneRendererError,

    #[error("Unable to lock Mutex for retrieving the SDL_Renderer on main")]
    MutexError(#[from] PoisonError<MutexGuard<'static, Option<SDL_Renderer>>>),
}

pub fn get_renderer() -> Result<*mut SDL_Renderer, MainError> {
    let renderer = RENDERER.lock()?;
    if let Some(mut sdl_renderer) = *renderer {
        Ok(&raw mut sdl_renderer)
    } else {
        Err(MainError::NoneRendererError)
    }
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

fn render_ui() {}

fn render_frame_time_dialog() {}

fn hybrid_sleep(seconds: DurationMs) {}

fn main_loop() {}

// bool defaults to false
fn imgui_menu_item_w_shortcut(binding: GameBindings, selected: Option<bool>) {}

fn main() -> Result<(), Box<dyn Error>> {
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
        if main_window.is_none() {
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
            let mut static_render = RENDERER.lock()?;
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
                CStr::from_ptr(renderer_info.name).to_str()?
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

            println!("Initializing IMGUI_SDL");
            imgui_sdl::initialize(&mut imgui_context, renderer, 0, 0);

            println!("Locking renderer to init");
            match RENDERER.lock() {
                Ok(renderer_opt) => match *renderer_opt {
                    Some(mut static_renderer) => {
                        imgui_sdl::init_for_sdl_renderer(
                            &mut imgui_context,
                            window,
                            addr_of_mut!(static_renderer),
                        );
                    }
                    Option::None => {
                        panic!("Could not find a renderer to initialize");
                    }
                },
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
                // pb::show_message_box(
                //     SDL_MESSAGEBOX_ERROR,
                //     "Could not load game data",
                //     CString::from_str(&message).unwrap().as_ptr(),
                // );
                exit(1);
            }

            // fullscrn::init();

            SDL_ShowWindow(window);

            let do_restart = RESTART.load(Relaxed);
            // if do_restart {
            //     Ok(break);
            // }
        }
    }
}
