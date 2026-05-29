use crate::options::InputTypes::{GameController, Keyboard, Mouse};
use crate::translations;
use crate::translations::Msg;
use dear_imgui_rs::sys::{
    ImGuiSettingsHandler, igAddSettingsHandler, igGetCurrentContext, igImHashStr,
    igLoadIniSettingsFromDisk,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use sdl2::sys::SDL_GameControllerButton::*;
use sdl2::sys::SDL_KeyCode::*;
use sdl2::sys::*;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::{c_char, c_int};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{LazyLock, Mutex, OnceLock};

static SETTINGS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static OPTIONS: LazyLock<Mutex<OptionsStruct>> = LazyLock::new(|| {
    Mutex::new(OptionsStruct {
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
        // TODO: Implement translations
        // language: Setting::new("Language", translations::GetCurrentLanguage().ShortName),
        language: Setting::new("Language", "".to_string()),
        hide_cursor: Setting::new("Hide Cursor", false),
    })
});
static SHOW_DIALOG: AtomicBool = AtomicBool::new(false);

pub static CONTROL_WAITING_FOR_INPUT: Mutex<Option<GameInput>> = Mutex::new(Option::None);

pub const MIX_MAX_VOLUME: i32 = 100; // TODO: Is it 100?

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Menu {
    NewGame = 101,
    AboutPinball = 102,
    HighScores = 103,
    Exit = 105,
    Sounds = 201,
    Music = 202,
    SoundStereo = 203,
    HelpTopics = 301,
    LaunchBall = 401,
    PauseResumeGame = 402,
    FullScreen = 403,
    Demo = 404,
    SelectTable = 405,
    PlayerControls = 406,
    OnePlayer = 408,
    TwoPlayers = 409,
    ThreePlayers = 410,
    FourPlayers = 411,
    ShowMenu = 412,
    MaximumResolution = 500,
    R640x480 = 501,
    R800x600 = 502,
    R1024x768 = 503,
    WindowUniformScale = 600,
    WindowLinearFilter = 601,
    WindowIntegerScale = 602,
    Prefer3DPBGameData = 700,
}

#[repr(i32)]
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, FromPrimitive)]
pub enum InputTypes {
    None = 0,
    Keyboard,
    Mouse,
    GameController,
}

#[derive(Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
pub struct GameInput {
    pub input_type: InputTypes,
    pub value: i32,
}

impl Default for GameInput {
    fn default() -> Self {
        Self {
            input_type: InputTypes::None,
            value: -1,
        }
    }
}

impl GameInput {
    pub fn new(input_type: InputTypes, value: i32) -> Self {
        Self { input_type, value }
    }

    pub fn get_full_input_description(&self) -> String {
        todo!()
    }

    pub fn get_short_input_description(&self) -> String {
        todo!()
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, FromPrimitive)]
pub enum GameBindings {
    Min = 0,
    LeftFlipper,
    RightFlipper,
    Plunger,
    LeftTableBump,
    RightTableBump,
    BottomTableBump,
    NewGame,
    TogglePause,
    ToggleFullScreen,
    ToggleSounds,
    ToggleMusic,
    ShowControlDialog,
    ToggleMenuDisplay,
    Exit,
    Max,
}

pub fn get_int(name: &str, default_value: i32) -> i32 {
    let settings = get_setting(name, &default_value.to_string());
    settings.parse::<i32>().unwrap_or(default_value)
}

pub fn get_setting(key: &str, default_value: &str) -> String {
    let mut hash_map = SETTINGS.lock().unwrap();

    match hash_map.entry(key.to_string()) {
        Entry::Occupied(entry) => entry.get().clone(),
        Entry::Vacant(entry) => {
            let new_value = entry.insert(default_value.to_string());
            // TODO: Add Imgui check
            new_value.clone()
        }
    }
}

pub fn set_int(name: &str, data: i32) {
    set_setting(name, &data.to_string());
}

fn set_setting(key: &str, value: &String) {
    let mut hash_map = SETTINGS.lock().unwrap();

    hash_map.insert(key.to_string(), value.to_string());
    // TODO: Add imgui check
}

// Original does ~120 updates per second.
pub const MAX_UPS: i32 = 360;
pub const MAX_FPS: i32 = MAX_UPS;
pub const MIN_UPS: i32 = 60;
pub const DEF_UPS: i32 = 120;
pub const DEF_FPS: i32 = 60;
pub const MIN_FPS: i32 = MIN_UPS;
// Original uses 8 sound channels
pub const MAX_SOUND_CHANNELS: i32 = 32;
pub const MIN_SOUND_CHANNELS: i32 = 1;
pub const DEF_SOUND_CHANNELS: i32 = 8;

pub const MAX_VOLUME: i32 = MIX_MAX_VOLUME;
pub const MIN_VOLUME: i32 = 0;

pub const DEF_VOLUME: i32 = MAX_VOLUME;

pub trait OptionBase {
    fn load(&mut self);
    fn save(&mut self);
    fn reset(&mut self);
}

pub trait SettingValue: Clone {
    fn fetch(name: &str, default: Self) -> Self;
    fn store(name: &str, value: &Self);
}

impl SettingValue for i32 {
    fn fetch(name: &str, default: i32) -> Self {
        let map = SETTINGS.lock().unwrap();
        match map.get(name) {
            Some(value) => value.parse::<i32>().unwrap_or(default),
            Option::None => default,
        }
    }
    fn store(name: &str, value: &i32) {
        let mut map = SETTINGS.lock().unwrap();
        map.insert(name.to_string(), value.to_string());
    }
}

impl SettingValue for bool {
    fn fetch(name: &str, default: Self) -> Self {
        let map = SETTINGS.lock().unwrap();
        match map.get(name) {
            Some(value) => value.parse::<bool>().unwrap_or(default),
            Option::None => default,
        }
    }
    fn store(name: &str, value: &bool) {
        let mut map = SETTINGS.lock().unwrap();
        map.insert(name.to_string(), value.to_string());
    }
}

impl SettingValue for String {
    fn fetch(name: &str, default: Self) -> Self {
        let map = SETTINGS.lock().unwrap();
        match map.get(name) {
            Some(value) => value.to_string(),
            Option::None => default,
        }
    }

    fn store(name: &str, value: &String) {
        let mut map = SETTINGS.lock().unwrap();
        map.insert(name.to_string(), value.to_string());
    }
}

impl SettingValue for f32 {
    fn fetch(name: &str, default: Self) -> Self {
        let map = SETTINGS.lock().unwrap();
        match map.get(name) {
            Some(value) => value.parse::<f32>().unwrap_or(default),
            Option::None => default,
        }
    }

    fn store(name: &str, value: &f32) {
        let mut map = SETTINGS.lock().unwrap();
        map.insert(name.to_string(), value.to_string());
    }
}

pub struct Setting<T: SettingValue> {
    pub name: &'static str,
    pub default: T,
    pub value: T,
}

impl<T: SettingValue> Setting<T> {
    pub fn new(name: &'static str, default: T) -> Self {
        Self {
            name,
            value: default.clone(),
            default,
        }
    }
}

impl<T: SettingValue> OptionBase for Setting<T> {
    fn load(&mut self) {
        self.value = T::fetch(self.name, self.default.clone());
    }

    fn save(&mut self) {
        T::store(self.name, &self.value);
    }

    fn reset(&mut self) {
        self.value = self.default.clone();
    }
}

impl<T: SettingValue> Deref for Setting<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: SettingValue> DerefMut for Setting<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

struct ControlOption {
    name: &'static str,
    description: Msg,
    defaults: [GameInput; 3],
    inputs: [GameInput; 3],
}

impl ControlOption {
    pub fn new(
        name: &'static str,
        description: Msg,
        default_keyboard: GameInput,
        default_mouse: GameInput,
        default_controller: GameInput,
    ) -> Self {
        Self {
            name,
            description,
            defaults: [default_keyboard, default_mouse, default_controller],
            inputs: [default_keyboard, default_mouse, default_controller],
        }
    }
}

impl OptionBase for ControlOption {
    fn load(&mut self) {
        for (idx, input) in self.inputs.iter_mut().enumerate() {
            let name = format!("{} {}", self.name, idx);
            input.input_type = self.defaults[idx].input_type;
            input.value = get_int(&format!("{} input", name), self.defaults[idx].value);
        }
    }

    fn save(&mut self) {
        for (idx, input) in self.inputs.iter_mut().enumerate() {
            let name = String::from(self.name) + " " + &idx.to_string();
            set_int(&format!("{} type", name), input.input_type as i32);
            set_int(&format!("{} input", name), input.value);
        }
    }

    fn reset(&mut self) {
        self.inputs.copy_from_slice(&self.defaults);
    }
}

pub struct OptionsStruct {
    pub control_options: [ControlOption; GameBindings::Max as usize - 1],
    pub sounds: Setting<bool>,
    pub music: Setting<bool>,
    pub full_screen: Setting<bool>,
    pub players: Setting<i32>,
    pub resolution: Setting<i32>,
    pub ui_scale: Setting<f32>,
    pub uniform_scaling: Setting<bool>,
    pub linear_filtering: Setting<bool>,
    pub frames_per_second: Setting<i32>,
    pub updates_per_second: Setting<i32>,
    pub show_menu: Setting<bool>,
    pub uncapped_updates_per_second: Setting<bool>,
    pub sound_channels: Setting<i32>,
    pub hybrid_sleep: Setting<bool>,
    pub prefer_3dpb_game_data: Setting<bool>,
    pub integer_scaling: Setting<bool>,
    pub sound_volume: Setting<i32>,
    pub music_volume: Setting<i32>,
    pub sound_stereo: Setting<bool>,
    pub debug_overlay: Setting<bool>,
    pub debug_overlay_grid: Setting<bool>,
    pub debug_overlay_all_edges: Setting<bool>,
    pub debug_overlay_ball_position: Setting<bool>,
    pub debug_overlay_ball_edges: Setting<bool>,
    pub debug_overlay_collision_mask: Setting<bool>,
    pub debug_overlay_sprites: Setting<bool>,
    pub debug_overlay_sounds: Setting<bool>,
    pub debug_overlay_ball_depth_grid: Setting<bool>,
    pub debug_overlay_aabb: Setting<bool>,
    pub font_file_name: Setting<String>,
    pub language: Setting<String>,
    pub hide_cursor: Setting<bool>,
}

impl OptionsStruct {
    pub fn all_options_mut(&mut self) -> Vec<&mut dyn OptionBase> {
        let mut options: Vec<&mut dyn OptionBase> = Vec::with_capacity(50);

        for ctrl_option in self.control_options.iter_mut() {
            options.push(ctrl_option);
        }

        options.push(&mut self.sounds);
        options.push(&mut self.music);
        options.push(&mut self.full_screen);
        options.push(&mut self.players);
        options.push(&mut self.resolution);
        options.push(&mut self.ui_scale);
        options.push(&mut self.uniform_scaling);
        options.push(&mut self.linear_filtering);
        options.push(&mut self.frames_per_second);
        options.push(&mut self.updates_per_second);
        options.push(&mut self.show_menu);
        options.push(&mut self.uncapped_updates_per_second);
        options.push(&mut self.sound_channels);
        options.push(&mut self.hybrid_sleep);
        options.push(&mut self.prefer_3dpb_game_data);
        options.push(&mut self.integer_scaling);
        options.push(&mut self.sound_volume);
        options.push(&mut self.music_volume);
        options.push(&mut self.sound_stereo);
        options.push(&mut self.debug_overlay);
        options.push(&mut self.debug_overlay_grid);
        options.push(&mut self.debug_overlay_all_edges);
        options.push(&mut self.debug_overlay_ball_position);
        options.push(&mut self.debug_overlay_ball_edges);
        options.push(&mut self.debug_overlay_collision_mask);
        options.push(&mut self.debug_overlay_sprites);
        options.push(&mut self.debug_overlay_sounds);
        options.push(&mut self.debug_overlay_ball_depth_grid);
        options.push(&mut self.debug_overlay_aabb);
        options.push(&mut self.font_file_name);
        options.push(&mut self.language);
        options.push(&mut self.hide_cursor);

        options
    }

    pub fn load_all(&mut self) {
        for opt in self.all_options_mut() {
            opt.load();
        }
    }

    pub fn save_all(&mut self) {
        for opt in self.all_options_mut() {
            opt.save();
        }
    }

    pub fn reset_all(&mut self) {
        for opt in self.all_options_mut() {
            opt.reset();
        }
    }
}

// WARNING: This is reaching for bindings to stuff that isn't normally exposed by imgui-rs
// I am not going to bother (for now) to spend the time to implement this properly
// So this should be a 1:1 (esque) translation of the original code
// The code below IS UNSAFE!
pub unsafe fn init_primary() {
    unsafe {
        let im_context = igGetCurrentContext();
        let mut ini_handler: ImGuiSettingsHandler = mem::zeroed();

        ini_handler.TypeName = c"Pinball".as_ptr();
        ini_handler.TypeHash = igImHashStr(ini_handler.TypeName, 0, 0);

        ini_handler.ReadOpenFn = MyUserData_ReadOpen;
        ini_handler.ReadLineFn = MyUserData_ReadLine;
        ini_handler.WriteAllFn = MyUserData_WriteAll;

        igAddSettingsHandler(&mut ini_handler);

        if (*im_context).SettingsLoaded {
            igLoadIniSettingsFromDisk((*im_context).IO.IniFilename);
            (*im_context).SettingsLoaded = true;
        }

        if let Ok(mut options) = OPTIONS.lock() {
            options.load_all();
        }
        PostProcessOptions();
    }
}

pub fn init_secondary() {
    let max_res = fullscrn::GetMaxResolution();
    let Ok(options) = OPTIONS.lock();
    if (options.resolution.value >= 0 && options.resolution.value > max_res) {
        options.resolution = max_res;
    }
    if (options.resolution.value == -1) {
        fullscrn::SetResolution(max_res);
    } else {
        fullscrn::SetResolution(options.resolution.value);
    }
}

pub fn uninit() {
    let Ok(mut options) = OPTIONS.lock();
    options.language.value = translations::GetCurrentLanguage().ShortName;
    options.save_all();
}

pub fn get_input(row_name: &str, mut values: [GameInput; 3]) {
    for (index, input) in values.iter_mut().enumerate() {
        let name = format!("{} {}", row_name, index);
        let type_val = get_int(&format!("{} type", name), -1);
        let input_type: InputTypes = InputTypes::from_i32(type_val).unwrap();
        let value = get_int(&format!("{} input", name), -1);

        if (input_type <= GameController && value != -1) {
            *input = GameInput { input_type, value };
        }
    }
}

pub fn set_input(row_name: &str, mut values: [GameInput; 3]) {
    for (index, input) in values.iter_mut().enumerate() {
        let name = format!("{} {}", row_name, index);
        set_int(&format!("{} type", input.input_type as i32), -1);
        set_int(&format!("{} input", input.value), -1);
    }
}

// TODO: Implement all the unimplemented stuff
pub fn toggle(u_id_check_item: Menu) {
    // TODO: The fuck is it complaining about
    let Ok(mut options) = OPTIONS.lock() else {
        todo!()
    };

    match u_id_check_item {
        Menu::NewGame => {}
        Menu::AboutPinball => {}
        Menu::HighScores => {}
        Menu::Exit => {}
        Menu::Sounds => {
            *options.sounds = !(*options.sounds);
            return;
        }
        Menu::Music => {
            *options.music = !(*options.music);
            if !(*options.music) {
                midi::music_stop();
            } else {
                midi::music_play();
            }
            return;
        }
        Menu::SoundStereo => {
            *options.sound_stereo = !(*options.sound_stereo);
            return;
        }
        Menu::HelpTopics => {}
        Menu::LaunchBall => {}
        Menu::PauseResumeGame => {}
        Menu::FullScreen => {
            *options.full_screen = !(*options.full_screen);
            fullscrn::set_screen_mode(options.full_screen);
        }
        Menu::Demo => {}
        Menu::SelectTable => {}
        Menu::PlayerControls => {}
        Menu::OnePlayer | Menu::TwoPlayers | Menu::ThreePlayers | Menu::FourPlayers => {}
        Menu::ShowMenu => {
            *options.show_menu = !(*options.show_menu);
            fullsrcn::window_size_changed();
        }
        Menu::MaximumResolution | Menu::R640x480 | Menu::R800x600 | Menu::R1024x768 => {
            let mut restart = false;
            let new_resolution = u_id_check_item as i32 - Menu::R640x480 as i32;
            if u_id_check_item == Menu::MaximumResolution {
                restart = fullscrn::GetResolution() != fullscrn::GetMaxResolution();
                *options.resolution = -1;
            } else if new_resolution <= fullscrn::GetMaxResolution() {
                let mut current_resolution: i32;
                if (*options.resolution == -1) {
                    current_resolution = fullscrn::GetMaxResolution();
                } else {
                    current_resolution = fullscrn::GetResolution();
                }

                let restart = (new_resolution != current_resolution);
            }

            if restart {
                main::Restart();
            }
        }
        Menu::WindowUniformScale => {
            *options.uniform_scaling = !(*options.uniform_scaling);
            fullscrn::window_size_changed();
        }
        Menu::WindowLinearFilter => {
            *options.linear_filtering = !(*options.linear_filtering);
            render::recreate_screen_texture();
        }
        Menu::WindowIntegerScale => {
            *options.integer_scaling = !(*options.integer_scaling);
            fullscrn::window_size_changed();
        }
        Menu::Prefer3DPBGameData => {
            *options.prefer_3dpb_game_data = !(*options.prefer_3dpb_game_data);
            fullscrn::window_size_changed();
        }
    }
}

pub fn input_down(input: GameInput) {
    let mut wait_flag = CONTROL_WAITING_FOR_INPUT.lock().unwrap();
    *wait_flag = Some(input);
}

pub fn show_control_dialog() {
    let dialog_check = SHOW_DIALOG.load(SeqCst);
    if !dialog_check {
        let mut wait_flag = CONTROL_WAITING_FOR_INPUT.lock().unwrap();
        *wait_flag = Option::None;
        SHOW_DIALOG.store(true, SeqCst);
        let mut options = OPTIONS.lock().unwrap();
        for option in &mut options.control_options {
            option.save();
        }
    }
}

pub fn render_control_dialog() {
    let dialog_check = SHOW_DIALOG.load(SeqCst);
    if !dialog_check {
        return;
    }

    // TODO: ImGui stuff

    let dialog_check = SHOW_DIALOG.load(SeqCst);
    if !dialog_check {
        return;
    }
}

pub fn MapGameInput(key: GameInput) -> Vec<GameBindings> {
    let mut result: Vec<GameBindings> = Vec::new();
    let mut options = OPTIONS.lock().unwrap();

    for input_id in GameBindings::Min as i32..GameBindings::Max as i32 {
        for inputValue in options.control_options[input_id as usize].inputs {
            if key == inputValue {
                result.push(GameBindings::from_i32(input_id).unwrap());
                break;
            }
        }
    }

    result
}

pub fn reset_all_options() {
    let mut options = OPTIONS.lock().unwrap();
    options.reset_all();
    todo!(); //    PostProcessOptions();
}
