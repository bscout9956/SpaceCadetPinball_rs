// Important: This module will be used to
// define the structs for the game state as
// to stop using static/mutexes everywhere.
// This change will be done over the course of multiple commits iteratively.
// Until this notice is removed, it may not hold all possible states.

use crate::fullscrn::ResolutionInfo;
use crate::gdrv::GdrvBitmap8;
use crate::group_data::DatFile;
use crate::options::InputTypes::{GameController, Keyboard, Mouse};
use crate::options::{
    ControlOption, DEF_FPS, DEF_SOUND_CHANNELS, DEF_UPS, DEF_VOLUME, GameInput, OptionsStruct,
    Setting,
};
use crate::pb::GameModes;
use crate::render::RenderSprite;
use crate::t_pinball_table::TPinballTable;
use crate::t_textbox::TTextBox;
use crate::translations::Msg;
use crate::{Duration, SdlRendererPtr, translations};
use sdl2::sys::SDL_GameControllerButton::{
    SDL_CONTROLLER_BUTTON_A, SDL_CONTROLLER_BUTTON_BACK, SDL_CONTROLLER_BUTTON_DPAD_LEFT,
    SDL_CONTROLLER_BUTTON_DPAD_RIGHT, SDL_CONTROLLER_BUTTON_DPAD_UP,
    SDL_CONTROLLER_BUTTON_LEFTSHOULDER, SDL_CONTROLLER_BUTTON_RIGHTSHOULDER,
    SDL_CONTROLLER_BUTTON_START,
};
use sdl2::sys::SDL_KeyCode::{
    SDLK_ESCAPE, SDLK_F2, SDLK_F3, SDLK_F4, SDLK_F5, SDLK_F6, SDLK_F8, SDLK_F9, SDLK_PERIOD,
    SDLK_SLASH, SDLK_SPACE, SDLK_UP, SDLK_x, SDLK_z,
};
use sdl2::sys::{SDL_BUTTON_LEFT, SDL_BUTTON_RIGHT, SDL_BUTTON_X1, SDL_BUTTON_X2};
use std::sync::{Arc, Mutex};

pub struct PinballState {
    pub main_state: MainState,
    pub pb_game_state: PbGameState,
    pub options_state: OptionsState,
    pub fullscrn_state: FullscrnState,
    pub render_state: RenderState,
}

pub struct PbGameState {
    pub ball_max_speed: f32,
    pub ball_half_radius: f32,
    pub ball_to_ball_collision_distance: f32,
    pub cheat_mode: bool,
    pub credits_active: bool,
    pub demo_mode: bool,
    pub full_tilt_mode: bool,
    pub full_tilt_demo_mode: bool,
    pub main_table: Option<TPinballTable>,
    pub time_ticks: usize,
    pub time_next: f32,
    pub time_now: f32,
    pub quick_flag: bool,
    pub game_mode: GameModes,
    pub record_table: Option<Arc<DatFile>>,
    pub dat_file_name: String,
    pub base_path: String,
    pub idle_timer_ms: f32,
    pub miss_text_box: Option<TTextBox>,
}

impl PbGameState {
    fn new() -> Self {
        Self {
            ball_max_speed: 0.0,
            ball_half_radius: 0.0,
            ball_to_ball_collision_distance: 0.0,
            full_tilt_mode: false,
            full_tilt_demo_mode: false,
            cheat_mode: false,
            credits_active: false,
            demo_mode: false,
            main_table: None,
            time_ticks: 0,
            time_next: 0.0,
            time_now: 0.0,
            quick_flag: false,
            game_mode: GameModes::GameOver,
            record_table: None,
            dat_file_name: String::new(),
            base_path: String::new(),
            idle_timer_ms: 0.0f32,
            miss_text_box: None,
        }
    }
}

pub struct RenderState {
    pub sprite_list: Vec<RenderSprite>,
    pub ball_list: Vec<RenderSprite>,
    pub v_screen: Option<GdrvBitmap8>,
}

impl RenderState {
    fn new() -> Self {
        Self {
            sprite_list: Vec::new(),
            ball_list: Vec::new(),
            v_screen: None,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
pub struct MainState {
    pub b_quit: bool,
    pub mouse_down: bool,
    pub has_focus: bool,
    pub return_value: i32,
    pub single_step: bool,
    pub target_frametime: Duration<1_000_000_000>,
    pub cursor_idle_counter: i32,
    pub fps_details: String,
    pub last_mouse_x: i32,
    pub last_mouse_y: i32,
    pub no_time_loss: bool,
    pub demo_active: bool,
    pub launch_ball_enabled: bool,
    pub high_scores_enabled: bool,
    pub gfr_display: Vec<f32>,
    pub prev_sdl_error: String,
    pub restart: bool,
    pub show_about_dialog: bool,
    pub show_imgui_demo: bool,
    pub show_sprite_viewer: bool,
    pub show_exit_popup: bool,
    pub renderer: Option<SdlRendererPtr>,
    pub update_to_frame_ratio: f64,
    pub disp_frame_rate: bool,
    pub disp_gr_history: bool,
    pub activated: bool,
    pub main_menu_height: i32,
}

impl MainState {
    fn new() -> MainState {
        Self {
            b_quit: false,
            mouse_down: false,
            has_focus: true,
            return_value: 0,
            single_step: false,
            target_frametime: Duration(0),
            cursor_idle_counter: 0,
            fps_details: String::new(),
            last_mouse_x: 0,
            last_mouse_y: 0,
            no_time_loss: false,
            demo_active: false,
            launch_ball_enabled: true,
            high_scores_enabled: true,
            gfr_display: Vec::new(),
            prev_sdl_error: String::new(),
            restart: false,
            show_about_dialog: false,
            show_imgui_demo: false,
            show_sprite_viewer: false,
            show_exit_popup: false,
            renderer: None,
            update_to_frame_ratio: 0.0,
            disp_frame_rate: false,
            disp_gr_history: false,
            activated: false,
            main_menu_height: 0,
        }
    }

    pub fn update_fps_details(&mut self, value: &str) {
        self.fps_details = value.to_string();
    }

    pub fn update_mouse_xy(&mut self, x: i32, y: i32) {
        self.last_mouse_x = x;
        self.last_mouse_y = y;
    }
}

impl PinballState {
    pub fn new() -> PinballState {
        Self {
            main_state: MainState::new(),
            pb_game_state: PbGameState::new(),
            options_state: OptionsState::new(),
            fullscrn_state: FullscrnState::new(),
            render_state: RenderState::new(),
        }
    }
}

pub struct OptionsState {
    pub show_dialog: bool,
    pub control_waiting_for_input: Option<GameInput>,
    pub options: OptionsStruct,
}

impl OptionsState {
    pub fn new() -> OptionsState {
        Self {
            show_dialog: false,
            control_waiting_for_input: None,
            options: get_default_options(),
        }
    }
}

pub struct FullscrnState {
    pub scale_x: f32,
    pub scale_y: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub resolution_array: [ResolutionInfo; 3],
    pub screen_mode: bool,
    pub display_changed: bool,
    pub resolution: i32,
}
impl FullscrnState {
    pub fn new() -> FullscrnState {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            resolution_array: [
                ResolutionInfo {
                    screen_width: 640,
                    screen_height: 480,
                    table_width: 600,
                    table_height: 416,
                    resolution_menu_id: 501,
                },
                ResolutionInfo {
                    screen_width: 800,
                    screen_height: 600,
                    table_width: 752,
                    table_height: 520,
                    resolution_menu_id: 502,
                },
                ResolutionInfo {
                    screen_width: 1024,
                    screen_height: 768,
                    table_width: 960,
                    table_height: 666,
                    resolution_menu_id: 503,
                },
            ],
            screen_mode: false,
            display_changed: false,
            resolution: 0,
        }
    }
}

fn get_default_options() -> OptionsStruct {
    OptionsStruct {
        control_options: [
            ControlOption::new(
                "Left Flipper key",
                Msg::KeymapperFlipperL,
                GameInput::new(Keyboard, SDLK_z as i32),
                GameInput::new(Mouse, SDL_BUTTON_LEFT as i32),
                GameInput::new(GameController, SDL_CONTROLLER_BUTTON_LEFTSHOULDER as i32),
            ),
            ControlOption::new(
                "Right Flipper key",
                Msg::KeymapperFlipperR,
                GameInput::new(Keyboard, SDLK_SLASH as i32),
                GameInput::new(Mouse, SDL_BUTTON_RIGHT as i32),
                GameInput::new(GameController, SDL_CONTROLLER_BUTTON_RIGHTSHOULDER as i32),
            ),
            ControlOption::new(
                "Plunger key",
                Msg::KeymapperPlunger,
                GameInput::new(Keyboard, SDLK_SPACE as i32),
                GameInput::new(Mouse, SDL_BUTTON_LEFT as i32),
                GameInput::new(GameController, SDL_CONTROLLER_BUTTON_A as i32),
            ),
            ControlOption::new(
                "Left Table Bump key",
                Msg::KeymapperBumpLeft,
                GameInput::new(Keyboard, SDLK_x as i32),
                GameInput::new(Mouse, SDL_BUTTON_X1 as i32),
                GameInput::new(GameController, SDL_CONTROLLER_BUTTON_DPAD_LEFT as i32),
            ),
            ControlOption::new(
                "Right Table Bump key",
                Msg::KeymapperBumpRight,
                GameInput::new(Keyboard, SDLK_PERIOD as i32),
                GameInput::new(Mouse, SDL_BUTTON_X2 as i32),
                GameInput::new(GameController, SDL_CONTROLLER_BUTTON_DPAD_RIGHT as i32),
            ),
            ControlOption::new(
                "Bottom Table Bump key",
                Msg::KeymapperBumpBottom,
                GameInput::new(Keyboard, SDLK_UP as i32),
                GameInput::new(Mouse, SDL_BUTTON_X2 as i32 + 1),
                GameInput::new(GameController, SDL_CONTROLLER_BUTTON_DPAD_UP as i32),
            ),
            ControlOption::new(
                "New Game",
                Msg::Menu1NewGame,
                GameInput::new(Keyboard, SDLK_F2 as i32),
                GameInput::default(),
                GameInput::default(),
            ),
            ControlOption::new(
                "Toggle Pause",
                Msg::Menu1PauseResumeGame,
                GameInput::new(Keyboard, SDLK_F3 as i32),
                GameInput::default(),
                GameInput::new(GameController, SDL_CONTROLLER_BUTTON_START as i32),
            ),
            ControlOption::new(
                "Toggle FullScreen",
                Msg::Menu1FullScreen,
                GameInput::new(Keyboard, SDLK_F4 as i32),
                GameInput::default(),
                GameInput::default(),
            ),
            ControlOption::new(
                "Toggle Sounds",
                Msg::Menu1Sounds,
                GameInput::new(Keyboard, SDLK_F5 as i32),
                GameInput::default(),
                GameInput::default(),
            ),
            ControlOption::new(
                "Toggle Music",
                Msg::Menu1Music,
                GameInput::new(Keyboard, SDLK_F6 as i32),
                GameInput::default(),
                GameInput::default(),
            ),
            ControlOption::new(
                "Show Control Dialog",
                Msg::Menu1PlayerControls,
                GameInput::new(Keyboard, SDLK_F8 as i32),
                GameInput::default(),
                GameInput::default(),
            ),
            ControlOption::new(
                "Toggle Menu Display",
                Msg::Menu1ToggleShowMenu,
                GameInput::new(Keyboard, SDLK_F9 as i32),
                GameInput::default(),
                GameInput::default(),
            ),
            ControlOption::new(
                "Exit",
                Msg::Menu1Exit,
                GameInput::new(Keyboard, SDLK_ESCAPE as i32),
                GameInput::default(),
                GameInput::new(GameController, SDL_CONTROLLER_BUTTON_BACK as i32),
            ),
        ],
        sounds: Setting::new("Sounds", true),
        music: Setting::new("Music", false),
        full_screen: Setting::new("FullScreen", false),
        players: Setting::new("Players", 1),
        resolution: Setting::new("Screen Resolution", -1),
        ui_scale: Setting::new("UI Scale", 1.0),
        uniform_scaling: Setting::new("Uniform scaling", true),
        linear_filtering: Setting::new("Linear Filtering", true),
        frames_per_second: Setting::new("Frames per Second", DEF_FPS),
        updates_per_second: Setting::new("Updates per Second", DEF_UPS),
        show_menu: Setting::new("ShowMenu", true),
        uncapped_updates_per_second: Setting::new("Uncapped Updates Per Second", false),
        sound_channels: Setting::new("Sound Channels", DEF_SOUND_CHANNELS),
        hybrid_sleep: Setting::new("HybridSleep", false),
        prefer_3dpb_game_data: Setting::new("Prefer 3DPB Game Data", false),
        integer_scaling: Setting::new("Integer Scaling", false),
        sound_volume: Setting::new("Sound Volume", DEF_VOLUME),
        music_volume: Setting::new("Music Volume", DEF_VOLUME),
        sound_stereo: Setting::new("Stereo Sound Effects", false),
        debug_overlay: Setting::new("Debug Overlay", false),
        debug_overlay_grid: Setting::new("Debug Overlay Grid", true),
        debug_overlay_all_edges: Setting::new("Debug Overlay All Edges", true),
        debug_overlay_ball_position: Setting::new("Debug Overlay Ball Position", true),
        debug_overlay_ball_edges: Setting::new("Debug Overlay Ball Edges", true),
        debug_overlay_collision_mask: Setting::new("Debug Overlay Collision Mask", true),
        debug_overlay_sprites: Setting::new("Debug Overlay Sprites", true),
        debug_overlay_sounds: Setting::new("Debug Overlay Sounds", true),
        debug_overlay_ball_depth_grid: Setting::new("Debug Overlay Ball Depth Grid", true),
        debug_overlay_aabb: Setting::new("Debug Overlay AABB", true),
        font_file_name: Setting::new("FontFileName", "".to_string()),
        language: Setting::new(
            "Language",
            translations::get_current_language()
                .unwrap()
                .short_name
                .to_string(),
        ),
        hide_cursor: Setting::new("Hide Cursor", false),
    }
}
