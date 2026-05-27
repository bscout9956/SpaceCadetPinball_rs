use std::cmp::PartialEq;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::c_char;
use std::ops::{Deref, DerefMut, Not};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::{LazyLock, Mutex, OnceLock};

static SETTINGS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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
    pub fn new(input_type: InputTypes) -> Self {
        Self {
            input_type: InputTypes::None,
            value: -1,
        }
    }

    pub fn new_type_value(input_type: InputTypes, value: i32) -> Self {
        Self { input_type, value }
    }
}

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

impl GameBindings {
    pub const MIN: GameBindings = GameBindings::LeftFlipper;
    pub const MAX: GameBindings = GameBindings::Exit;
    pub const COUNT: usize = (Self::MAX as usize) + 1;

    pub fn increment(&mut self) {
        let cur_val = *self as usize;
        if cur_val < Self::MAX as usize {
            *self = match cur_val + 1 {
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
                _ => GameBindings::NewGame, // TODO: Correct???
            }
        }
    }
}

pub struct Options {
    pub options: OnceLock<Mutex<OptionsStruct>>,
    pub all_options: OnceLock<Mutex<Vec<OptionBase>>>,

    // TODO: Do I want &str or String?
    pub settings: OnceLock<Mutex<HashMap<String, String>>>,
    show_dialog: AtomicBool,
    control_waiting_for_input: OnceLock<Mutex<GameInput>>,
}

pub const MIX_MAX_VOLUME: i32 = 1; // TODO: IDK?

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
    set_settings(name, &data.to_string());
}

fn set_settings(key: &str, value: &String) {
    let mut hash_map = SETTINGS.lock().unwrap();

    hash_map.insert(key.to_string(), value.to_string());
    // TODO: Add imgui check
}

impl Options {
    // Original does ~120 updates per second.
    pub const MAX_UPS: i32 = 360;
    pub const MAX_FPS: i32 = Self::MAX_UPS;
    pub const MIN_UPS: i32 = 60;
    pub const DEF_UPS: i32 = 120;
    pub const DEF_FPS: i32 = 60;
    pub const MIN_FPS: i32 = Self::MIN_UPS;
    // Original uses 8 sound channels
    pub const MAX_SOUND_CHANNELS: i32 = 32;
    pub const MIN_SOUND_CHANNELS: i32 = 1;
    pub const DEF_SOUND_CHANNELS: i32 = 8;

    pub const MAX_VOLUME: i32 = MIX_MAX_VOLUME;
    pub const MIN_VOLUME: i32 = 0;

    pub const DEF_VOLUME: i32 = Self::MAX_VOLUME;
}

pub struct OptionBase {
    name: *mut c_char,
}

pub trait OptionBaseBehavior {
    fn load(&mut self);
    fn save(&mut self);
    fn reset(&mut self);
}

pub struct OptionBaseT<T> {
    name: &'static str,
    default_value: T,
    value: T,
}

impl<T: Clone> OptionBaseBehavior for OptionBaseT<T> {
    fn load() {
        todo!()
    }

    fn save() {
        todo!()
    }

    fn reset(&mut self) {
        self.value = self.default_value.clone();
    }
}

impl<T> Deref for OptionBaseT<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for OptionBaseT<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Clone> OptionBaseT<T> {
    pub fn new(name: &'static str, default_value: T) -> Self {
        let value = default_value.clone();
        Self {
            name,
            default_value,
            value,
        }
    }
}

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
    description: &'static str, // TODO In the original it's a enum class with type int????
    defaults: [GameInput; 3],
    inputs: [GameInput; 3],
}

impl ControlOption {
    pub fn new(
        name: &'static str,
        description: &'static str,
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
        for (idx, input) in &self.inputs.iter().enumerate() {
            let name = String::from(self.name) + " " + idx.to_string();
            // TODO: Fix conversion
            input.input_type =
                get_int(&(name + " type"), self.defaults[idx].input_type as i32) as InputTypes;
            input.value = get_int(&(name + " input"), self.defaults[idx].value);
        }
    }

    fn save(&mut self) {
        for (idx, input) in self.inputs.iter().enumerate() {
            let name = String::from(self.name) + " " + &idx.to_string();
            set_int(&(name + " type"), input.input_type as i32);
            set_int(&(name + " input"), input.value);
        }
    }

    fn reset(&mut self) {
        todo!();
    }
}

impl OptionBaseBehavior for IntOption {
    fn load(&mut self) {
        self.value = get_int(self.name, self.default_value);
    }

    fn save(&self) {
        set_int(self.name, self.value)
    }

    fn reset(&mut self) {
        self.value = self.default_value;
        self.save();
    }
}

struct OptionsStruct {
    pub key: [ControlOption; GameBindings::COUNT],
}
