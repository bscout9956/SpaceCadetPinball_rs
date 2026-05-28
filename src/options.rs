use crate::options::InputTypes::{GameController, Keyboard, Mouse};
use crate::translations;
use crate::translations::Msg;
use imgui::sys::{ImGuiSettingsHandler, igImHashStr, igLoadIniSettingsFromDisk};
use sdl2::sys::SDL_GameControllerButton::*;
use sdl2::sys::SDL_KeyCode::*;
use sdl2::sys::*;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::c_char;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::{LazyLock, Mutex, OnceLock};

static SETTINGS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static OPTIONS: LazyLock<Mutex<OptionsStruct>> = LazyLock::new(|| {
    Mutex::new(OptionsStruct {
        key: [
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
                GameInput::new_empty(),
                GameInput::new_empty(),
            ),
            ControlOption::new(
                "Toggle Pause",
                Msg::Menu1PauseResumeGame,
                GameInput::new(Keyboard, SDLK_F3 as i32),
                GameInput::new_empty(),
                GameInput::new(GameController, SDL_CONTROLLER_BUTTON_START as i32),
            ),
            ControlOption::new(
                "Toggle FullScreen",
                Msg::Menu1FullScreen,
                GameInput::new(Keyboard, SDLK_F4 as i32),
                GameInput::new_empty(),
                GameInput::new_empty(),
            ),
            ControlOption::new(
                "Toggle Sounds",
                Msg::Menu1Sounds,
                GameInput::new(Keyboard, SDLK_F5 as i32),
                GameInput::new_empty(),
                GameInput::new_empty(),
            ),
            ControlOption::new(
                "Toggle Music",
                Msg::Menu1Music,
                GameInput::new(Keyboard, SDLK_F6 as i32),
                GameInput::new_empty(),
                GameInput::new_empty(),
            ),
            ControlOption::new(
                "Show Control Dialog",
                Msg::Menu1PlayerControls,
                GameInput::new(Keyboard, SDLK_F8 as i32),
                GameInput::new_empty(),
                GameInput::new_empty(),
            ),
            ControlOption::new(
                "Toggle Menu Display",
                Msg::Menu1ToggleShowMenu,
                GameInput::new(Keyboard, SDLK_F9 as i32),
                GameInput::new_empty(),
                GameInput::new_empty(),
            ),
            ControlOption::new(
                "Exit",
                Msg::Menu1Exit,
                GameInput::new(Keyboard, SDLK_ESCAPE as i32),
                GameInput::new_empty(),
                GameInput::new(GameController, SDL_CONTROLLER_BUTTON_BACK as i32),
            ),
        ],
        sounds: BoolOption::new("Sounds", true),
        music: BoolOption::new("Music", false),
        full_screen: BoolOption::new("FullScreen", false),
        players: IntOption::new("Players", 1),
        resolution: IntOption::new("Screen Resolution", -1),
        ui_scale: FloatOption::new("UI Scale", 1.0),
        uniform_scaling: BoolOption::new("Uniform scaling", true),
        linear_filtering: BoolOption::new("Linear Filtering", true),
        frames_per_second: IntOption::new("Frames per Second", DEF_FPS),
        updates_per_second: IntOption::new("Updates per Second", DEF_UPS),
        show_menu: BoolOption::new("ShowMenu", true),
        uncapped_updates_per_second: BoolOption::new("Uncapped Updates Per Second", false),
        sound_channels: IntOption::new("Sound Channels", DEF_SOUND_CHANNELS),
        hybrid_sleep: BoolOption::new("HybridSleep", false),
        prefer_3dpb_game_data: BoolOption::new("Prefer 3DPB Game Data", false),
        integer_scaling: BoolOption::new("Integer Scaling", false),
        sound_volume: IntOption::new("Sound Volume", DEF_VOLUME),
        music_volume: IntOption::new("Music Volume", DEF_VOLUME),
        sound_stereo: BoolOption::new("Stereo Sound Effects", false),
        debug_overlay: BoolOption::new("Debug Overlay", false),
        debug_overlay_grid: BoolOption::new("Debug Overlay Grid", true),
        debug_overlay_all_edges: BoolOption::new("Debug Overlay All Edges", true),
        debug_overlay_ball_position: BoolOption::new("Debug Overlay Ball Position", true),
        debug_overlay_ball_edges: BoolOption::new("Debug Overlay Ball Edges", true),
        debug_overlay_collision_mask: BoolOption::new("Debug Overlay Collision Mask", true),
        debug_overlay_sprites: BoolOption::new("Debug Overlay Sprites", true),
        debug_overlay_sounds: BoolOption::new("Debug Overlay Sounds", true),
        debug_overlay_ball_depth_grid: BoolOption::new("Debug Overlay Ball Depth Grid", true),
        debug_overlay_aabb: BoolOption::new("Debug Overlay AABB", true),
        font_file_name: StringOption::new("FontFileName", ""),
        // TODO: Implement translations
        language: StringOption::new("Language", translations::GetCurrentLanguage().ShortName),
        hide_cursor: BoolOption::new("Hide Cursor", false),
    })
});
static ALL_OPTIONS: LazyLock<Mutex<Vec<OptionBase>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static SHOW_DIALOG: AtomicBool = AtomicBool::new(false);

static CONTROL_WAITING_FOR_INPUT: LazyLock<Mutex<GameInput>> =
    LazyLock::new(|| Mutex::new(GameInput::new_empty()));

pub const MIX_MAX_VOLUME: i32 = 100; // TODO: Is it 100?

#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputTypes {
    None = 0,
    Keyboard,
    Mouse,
    GameController,
}

#[derive(Clone, Copy)]
pub struct GameInput {
    pub input_type: InputTypes,
    pub value: i32,
}

impl PartialEq for GameInput {
    fn eq(&self, other: &Self) -> bool {
        self.input_type == other.input_type && self.value == other.value
    }
}

impl GameInput {
    pub fn new_empty() -> Self {
        Self {
            input_type: InputTypes::None,
            value: -1,
        }
    }

    pub fn new(input_type: InputTypes, value: i32) -> Self {
        Self { input_type, value }
    }
}

// TODO: Make it back into an enum?
#[repr(usize)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum GameBindings {
    LeftFlipper = 0,
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
}

// TODO: Make it back into an enum?
impl GameBindings {
    pub const MIN: GameBindings = GameBindings::LeftFlipper;
    pub const MAX: GameBindings = GameBindings::Exit;
    pub const COUNT: usize = (Self::MAX as usize) + 1;

    fn get_value(&mut self) -> usize {
        let cur_val = *self as usize;
        cur_val
    }

    fn get_enum(value: usize) -> GameBindings {
        match value {
            1 => GameBindings::LeftFlipper,
            2 => GameBindings::RightFlipper,
            3 => GameBindings::Plunger,
            4 => GameBindings::LeftTableBump,
            5 => GameBindings::RightTableBump,
            6 => GameBindings::BottomTableBump,
            7 => GameBindings::NewGame,
            8 => GameBindings::TogglePause,
            9 => GameBindings::ToggleFullScreen,
            10 => GameBindings::ToggleSounds,
            11 => GameBindings::ToggleMusic,
            12 => GameBindings::ShowControlDialog,
            13 => GameBindings::ToggleMenuDisplay,
            14 => GameBindings::Exit,
            _ => GameBindings::NewGame,
        }
    }

    pub fn increment(&mut self) {
        let cur_val = *self as usize;
        if cur_val < Self::MAX as usize {
            *self = Self::get_enum(cur_val + 1);
        }
    }
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

pub fn set_float(name: &str, data: f32) {
    set_setting(name, &data.to_string());
}

pub fn get_float(name: &str, default_value: f32) -> f32 {
    let value = get_setting(name, &default_value.to_string());
    value.parse::<f32>().unwrap_or(default_value)
}

pub fn set_bool(name: &str, value: bool) {
    set_setting(name, &value.to_string());
}

pub fn get_bool(name: &str, default_value: bool) -> bool {
    let value = get_setting(name, &default_value.to_string());
    value.parse::<bool>().unwrap_or(default_value)
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

pub struct OptionBase {
    name: &'static str,
}

pub trait OptionBaseBehavior {
    fn load(&mut self);
    fn save(&mut self);
    fn reset(&mut self);
}

// pub struct OptionBaseT<T> {
//     name: &'static str,
//     default_value: T,
//     value: T,
// }

// impl<T: Clone> OptionBaseBehavior for OptionBaseT<T> {
//     fn load(&mut self) {
//         todo!()
//     }
//
//     fn save(&mut self) {
//         todo!()
//     }
//
//     fn reset(&mut self) {
//         self.value = self.default_value.clone();
//     }
// }
//
// impl<T> Deref for OptionBaseT<T> {
//     type Target = T;
//
//     fn deref(&self) -> &Self::Target {
//         &self.value
//     }
// }
//
// impl<T> DerefMut for OptionBaseT<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.value
//     }
// }
//
// impl<T: Clone> OptionBaseT<T> {
//     pub fn new(name: &'static str, default_value: T) -> Self {
//         let value = default_value.clone();
//         Self {
//             name,
//             default_value,
//             value,
//         }
//     }
// }

struct IntOption {
    name: &'static str,
    default_value: i32,
    value: i32,
}

impl IntOption {
    pub fn new(name: &'static str, default_value: i32) -> Self {
        Self {
            name,
            default_value,
            value: default_value,
        }
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

impl OptionBaseBehavior for ControlOption {
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

impl OptionBaseBehavior for IntOption {
    fn load(&mut self) {
        self.value = get_int(self.name, self.default_value);
    }

    fn save(&mut self) {
        set_int(self.name, self.value)
    }

    fn reset(&mut self) {
        self.value = self.default_value;
        self.save();
    }
}

impl Deref for IntOption {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for IntOption {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct StringOption {
    name: &'static str,
    default_value: String,
    value: String,
}

impl StringOption {
    pub fn new(name: &'static str, default_value: &str) -> Self {
        Self {
            name,
            default_value: String::from(default_value),
            value: String::from(default_value),
        }
    }
}

impl OptionBaseBehavior for StringOption {
    fn load(&mut self) {
        self.value = get_setting(self.name, &self.default_value);
    }

    fn save(&mut self) {
        set_setting(&self.name, &self.default_value);
    }

    fn reset(&mut self) {
        self.value = self.default_value.clone();
        self.save();
    }
}

impl Deref for StringOption {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for StringOption {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct FloatOption {
    name: &'static str,
    default_value: f32,
    value: f32,
}

impl FloatOption {
    pub fn new(name: &'static str, default_value: f32) -> Self {
        Self {
            name,
            default_value,
            value: default_value,
        }
    }
}

impl OptionBaseBehavior for FloatOption {
    fn load(&mut self) {
        self.value = get_float(self.name, self.default_value);
    }

    fn save(&mut self) {
        set_float(self.name, self.value);
    }

    fn reset(&mut self) {
        self.value = self.default_value;
        self.save();
    }
}

impl Deref for FloatOption {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for FloatOption {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct BoolOption {
    name: &'static str,
    default_value: bool,
    value: bool,
}

impl OptionBaseBehavior for BoolOption {
    fn load(&mut self) {
        self.value = get_bool(self.name, self.default_value);
    }

    fn save(&mut self) {
        set_bool(self.name, self.value);
    }

    fn reset(&mut self) {
        self.value = self.default_value;
        self.save();
    }
}

impl BoolOption {
    pub fn new(name: &'static str, default_value: bool) -> Self {
        Self {
            name,
            default_value,
            value: default_value,
        }
    }
}

impl Deref for BoolOption {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for BoolOption {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub struct OptionsStruct {
    pub key: [ControlOption; GameBindings::COUNT],
    pub sounds: BoolOption,
    pub music: BoolOption,
    pub full_screen: BoolOption,
    pub players: IntOption,
    pub resolution: IntOption,
    pub ui_scale: FloatOption,
    pub uniform_scaling: BoolOption,
    pub linear_filtering: BoolOption,
    pub frames_per_second: IntOption,
    pub updates_per_second: IntOption,
    pub show_menu: BoolOption,
    pub uncapped_updates_per_second: BoolOption,
    pub sound_channels: IntOption,
    pub hybrid_sleep: BoolOption,
    pub prefer_3dpb_game_data: BoolOption,
    pub integer_scaling: BoolOption,
    pub sound_volume: IntOption,
    pub music_volume: IntOption,
    pub sound_stereo: BoolOption,
    pub debug_overlay: BoolOption,
    pub debug_overlay_grid: BoolOption,
    pub debug_overlay_all_edges: BoolOption,
    pub debug_overlay_ball_position: BoolOption,
    pub debug_overlay_ball_edges: BoolOption,
    pub debug_overlay_collision_mask: BoolOption,
    pub debug_overlay_sprites: BoolOption,
    pub debug_overlay_sounds: BoolOption,
    pub debug_overlay_ball_depth_grid: BoolOption,
    pub debug_overlay_aabb: BoolOption,
    pub font_file_name: StringOption,
    pub language: StringOption,
    pub hide_cursor: BoolOption,
}

// WARNING: This is reaching for bindings to stuff that isn't normally exposed by imgui-rs
// I am not going to bother (for now) to spend the time to implement this properly
// So this should be a 1:1 (esque) translation of the original code
// The code below IS UNSAFE!
pub fn init_primary() {
    let im_context = unsafe { imgui::sys::igGetCurrentContext() };

    let mut ini_handler: ImGuiSettingsHandler;
    ini_handler = ImGuiSettingsHandler::default();
    ini_handler.TypeName = c"Pinball".as_ptr();

    // TODO: Count null terminated \0 or not?
    ini_handler.TypeHash = unsafe {
        igImHashStr(
            ini_handler.TypeName,
            c"Pinball".count_bytes(),
            0xDEADBEEFu32,
        )
    };

    ini_handler.ReadOpenFn = MyUserData_ReadOpen;
    ini_handler.ReadLineFn = MyUserData_ReadLine;
    ini_handler.WriteAllFn = MyUserData_WriteAll;

    // TODO: What's push_back again? Enqueue? Enplace?
    im_context.SettingsHandlers.push_back(ini_handler);
    unsafe {
        if !(*im_context).SettingsLoaded {
            igLoadIniSettingsFromDisk((*im_context).IO.IniFilename);
            (*im_context).SettingsLoaded = true;
        }
    }
    
    // TODO "Unwrap" ALL_OPTIONS
    for opt in ALL_OPTIONS {
        opt.Load();
    }
    
    // TODO: Implement me
    PostProcessOptions();
}
