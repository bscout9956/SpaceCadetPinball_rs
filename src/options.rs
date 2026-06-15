use crate::options::InputTypes::{GameController, Keyboard, Mouse};
use crate::pinball_state::{OptionsState, PbGameState};
use crate::translations::Msg;
use crate::utils::clamp;
use crate::{fullscrn, midi, render, translations};
use dear_imgui_rs::sys::{
    ImGuiContext, ImGuiSettingsHandler, ImGuiTextBuffer, ImGuiTextBuffer_append,
    ImGuiTextBuffer_appendf, igAddSettingsHandler, igGetCurrentContext, igImHashStr,
    igLoadIniSettingsFromDisk,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use sdl2::sys::SDL_GameControllerButton::*;
use sdl2::sys::SDL_KeyCode::*;
use sdl2::sys::*;
use std::cmp::{PartialEq, PartialOrd, max};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::{CStr, CString, c_char, c_void};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{LazyLock, Mutex};

static SETTINGS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static SHOW_DIALOG: AtomicBool = AtomicBool::new(false);

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
    pub unsafe fn get_full_input_description(&self) -> String {
        let mut prefix = String::new();
        match self.input_type {
            InputTypes::None => {
                return String::from("Unused");
            }
            Keyboard => {
                prefix = "Keyboard\n".to_string();
            }
            Mouse => {
                prefix = "Mouse\n".to_string();
            }
            GameController => {
                prefix = "GameController\n".to_string();
            }
        }
        unsafe {
            prefix
                + CStr::from_ptr(self.get_short_input_description())
                    .to_str()
                    .unwrap()
        }
    }

    pub unsafe fn get_short_input_description(&self) -> *const c_char {
        let mouse_buttons: [*const c_char; 6] = [
            std::ptr::null(),
            c"Left".as_ptr(),
            c"Middle".as_ptr(),
            c"Right".as_ptr(),
            c"X1".as_ptr(),
            c"X2".as_ptr(),
        ];

        let controller_buttons: [*const c_char; 21] = [
            c"A".as_ptr(),
            c"B".as_ptr(),
            c"X".as_ptr(),
            c"Y".as_ptr(),
            c"Back".as_ptr(),
            c"Guide".as_ptr(),
            c"Start".as_ptr(),
            c"LeftStick".as_ptr(),
            c"RightStick".as_ptr(),
            c"LeftShoulder".as_ptr(),
            c"RightShoulder".as_ptr(),
            c"DpUp".as_ptr(),
            c"DpDown".as_ptr(),
            c"DpLeft".as_ptr(),
            c"DpRight".as_ptr(),
            c"Misc1".as_ptr(),
            c"Paddle1".as_ptr(),
            c"Paddle2".as_ptr(),
            c"Paddle3".as_ptr(),
            c"Paddle4".as_ptr(),
            c"Touchpad".as_ptr(),
        ];

        match self.input_type {
            InputTypes::None => std::ptr::null(),
            Keyboard => unsafe { SDL_GetKeyName(self.value) },
            Mouse => {
                if self.value >= SDL_BUTTON_LEFT as i32 && self.value <= SDL_BUTTON_X2 as i32 {
                    mouse_buttons[self.value as usize]
                } else {
                    // VERIFY: Maybe we could just return string, we just need to make sure nothing else calls this necessarily
                    // TODO: Dangling pointer
                    CString::from_str(format!("MButton {}", self.value).as_str())
                        .unwrap_or_default()
                        .as_ptr()
                }
            }
            GameController => {
                if self.value >= SDL_CONTROLLER_BUTTON_A as i32
                    && self.value < std::cmp::min(SDL_CONTROLLER_BUTTON_MAX as i32, 21)
                {
                    controller_buttons[self.value as usize]
                } else {
                    // VERIFY: Maybe we could just return string?
                    // We just need to make sure nothing else calls this necessarily
                    // TODO: Dangling pointer
                    CString::from_str(format!("CButton {}", self.value).as_str())
                        .unwrap_or_default()
                        .as_ptr()
                }
            }
        }
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

#[derive(Ord, Eq, PartialEq, PartialOrd, Copy, Clone)]
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

pub struct ControlOption {
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

    pub unsafe fn get_shortcut_description(&self) -> String {
        let mut result: String = String::new();
        for input in self.inputs {
            if input.input_type != InputTypes::None {
                return unsafe {
                    CStr::from_ptr(input.get_short_input_description())
                        .to_str()
                        .unwrap_or_default()
                        .to_string()
                };
            }
        }
        result
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
pub unsafe fn init_primary(options_state: &mut OptionsState) {
    unsafe {
        let im_context = igGetCurrentContext();
        let mut ini_handler: ImGuiSettingsHandler = std::mem::zeroed();

        ini_handler.TypeName = c"Pinball".as_ptr();
        ini_handler.TypeHash = igImHashStr(ini_handler.TypeName, 0, 0);

        ini_handler.ReadOpenFn = Some(MyUserData_ReadOpen);
        ini_handler.ReadLineFn = Some(MyUserData_ReadLine);
        ini_handler.WriteAllFn = Some(MyUserData_WriteAll);

        igAddSettingsHandler(&mut ini_handler);

        if (*im_context).SettingsLoaded == false {
            igLoadIniSettingsFromDisk((*im_context).IO.IniFilename);
            (*im_context).SettingsLoaded = true;
        }

        options_state.options.load_all();
        post_process_options(options_state);
    }
}

pub fn init_secondary(options_state: &mut OptionsState, pb_game_state: &mut PbGameState) {
    let max_res = fullscrn::get_max_resolution(pb_game_state);

    if (options_state.options.resolution.value >= 0
        && options_state.options.resolution.value > max_res)
    {
        *options_state.options.resolution = max_res;
    }
    if (options_state.options.resolution.value == -1) {
        fullscrn::set_resolution(max_res, pb_game_state);
    } else {
        fullscrn::set_resolution(*options_state.options.resolution, pb_game_state);
    }
}

pub fn uninit(options_state: &mut OptionsState) {
    if let Some(cur_lang) = translations::get_current_language() {
        options_state.options.language.value = cur_lang.short_name.to_string();
        options_state.options.save_all();
    } else {
        println!("Unable to obtain current language info...");
    }
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
pub fn toggle(u_id_check_item: Menu, options_state: &mut OptionsState, pb_game_state: &mut PbGameState) {
    match u_id_check_item {
        Menu::NewGame => {}
        Menu::AboutPinball => {}
        Menu::HighScores => {}
        Menu::Exit => {}
        Menu::Sounds => {
            *options_state.options.sounds = !(*options_state.options.sounds);
            return;
        }
        Menu::Music => {
            *options_state.options.music = !(*options_state.options.music);
            if (*options_state.options.music) == false {
                midi::music_stop();
            } else {
                midi::music_play();
            }
            return;
        }
        Menu::SoundStereo => {
            *options_state.options.sound_stereo = !(*options_state.options.sound_stereo);
            return;
        }
        Menu::HelpTopics => {}
        Menu::LaunchBall => {}
        Menu::PauseResumeGame => {}
        Menu::FullScreen => {
            *options_state.options.full_screen = !(*options_state.options.full_screen);
            fullscrn::set_screen_mode(*options_state.options.full_screen);
        }
        Menu::Demo => {}
        Menu::SelectTable => {}
        Menu::PlayerControls => {}
        Menu::OnePlayer | Menu::TwoPlayers | Menu::ThreePlayers | Menu::FourPlayers => {}
        Menu::ShowMenu => {
            *options_state.options.show_menu = !(*options_state.options.show_menu);
            fullscrn::window_size_changed(options_state);
        }
        Menu::MaximumResolution | Menu::R640x480 | Menu::R800x600 | Menu::R1024x768 => {
            let mut restart = false;
            let new_resolution = u_id_check_item as i32 - Menu::R640x480 as i32;
            if u_id_check_item == Menu::MaximumResolution {
                restart = fullscrn::get_resolution() != fullscrn::get_max_resolution(pb_game_state);
                *options_state.options.resolution = -1;
            } else if new_resolution <= fullscrn::get_max_resolution(pb_game_state) {
                let mut current_resolution: i32;
                if (*options_state.options.resolution == -1) {
                    current_resolution = fullscrn::get_max_resolution(pb_game_state);
                } else {
                    current_resolution = fullscrn::get_resolution();
                }

                let restart = (new_resolution != current_resolution);
            }

            if restart {
                //TODO: restart_func();
            }
        }
        Menu::WindowUniformScale => {
            *options_state.options.uniform_scaling = !(*options_state.options.uniform_scaling);
            fullscrn::window_size_changed(options_state);
        }
        Menu::WindowLinearFilter => {
            *options_state.options.linear_filtering = !(*options_state.options.linear_filtering);
            render::recreate_screen_texture(options_state);
        }
        Menu::WindowIntegerScale => {
            *options_state.options.integer_scaling = !(*options_state.options.integer_scaling);
            fullscrn::window_size_changed(options_state);
        }
        Menu::Prefer3DPBGameData => {
            *options_state.options.prefer_3dpb_game_data =
                !(*options_state.options.prefer_3dpb_game_data);
            fullscrn::window_size_changed(options_state);
        }
    }
}

// pub fn input_down(input: GameInput) {
//     let mut wait_flag = CONTROL_WAITING_FOR_INPUT.lock().unwrap();
//     *wait_flag = Some(input);
// }

// pub fn show_control_dialog() {
//     let dialog_check = SHOW_DIALOG.load(SeqCst);
//     if !dialog_check {
//         let mut wait_flag = CONTROL_WAITING_FOR_INPUT.lock().unwrap();
//         *wait_flag = Option::None;
//         SHOW_DIALOG.store(true, SeqCst);
//         let mut options = OPTIONS.lock().unwrap();
//         for option in &mut options.control_options {
//             option.save();
//         }
//     }
// }

pub fn render_control_dialog() {
    let dialog_check = SHOW_DIALOG.load(SeqCst);
    if !dialog_check {
        return;
    }

    // TODO: ImGui stuff

    let dialog_check = SHOW_DIALOG.load(SeqCst);
    if !dialog_check {}
}

pub fn map_game_input(key: GameInput, options_state: &mut OptionsState) -> Vec<GameBindings> {
    let mut result: Vec<GameBindings> = Vec::new();

    for input_id in GameBindings::Min as i32..GameBindings::Max as i32 {
        for input_value in options_state.options.control_options[input_id as usize].inputs {
            if key == input_value {
                result.push(GameBindings::from_i32(input_id).unwrap());
                break;
            }
        }
    }

    result
}

pub fn reset_all_options(options_state: &mut OptionsState) {
    options_state.options.reset_all();
    post_process_options(options_state);
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn MyUserData_ReadLine(
    ctx: *mut ImGuiContext,
    handler: *mut ImGuiSettingsHandler,
    entry: *mut c_void,
    line: *const c_char,
) {
    unsafe {
        let kv_store = &mut *(entry as *mut HashMap<String, String>);
        let key_value = CStr::from_ptr(line).to_str().unwrap_or_default();
        if let Some(separator_pos) = key_value.find('=') {
            let key = &key_value[0..separator_pos];
            let value = &key_value[separator_pos + 1..];
            kv_store.insert(key.to_string(), value.to_string());
        }
    }
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn MyUserData_ReadOpen(
    ctx: *mut ImGuiContext,
    handler: *mut ImGuiSettingsHandler,
    name: *const c_char,
) -> *mut c_void {
    if name.eq(&c"Settings".as_ptr()) {
        let settings = SETTINGS.lock().unwrap();
        let mut clone_hash = settings.clone();
        // TODO: Dangling pointer
        return &raw mut clone_hash as *mut c_void;
    }
    std::ptr::null_mut()
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn MyUserData_WriteAll(
    ctx: *mut ImGuiContext,
    handler: *mut ImGuiSettingsHandler,
    buf: *mut ImGuiTextBuffer,
) {
    unsafe {
        ImGuiTextBuffer_appendf(buf, c"%s%s\n".as_ptr(), (*handler).TypeName, "Settings");
        let settings = SETTINGS.lock().unwrap();
        for setting in settings.iter() {
            ImGuiTextBuffer_appendf(buf, c"%s=%s\n".as_ptr(), setting.0, setting.1);
        }
        // VERIFY: str end?
        ImGuiTextBuffer_append(buf, c"\n".as_ptr(), std::ptr::null());
    }
}

// TODO Implement all the trash
pub fn post_process_options(options_state: &mut OptionsState) {
    // TODO: Pull this
    //main::ImIO.FontGlobalScale = *options_state.options.ui_scale;
    *options_state.options.frames_per_second = clamp(
        &options_state.options.frames_per_second.value,
        &MIN_FPS,
        &MAX_FPS,
    );
    *options_state.options.updates_per_second = clamp(
        &options_state.options.updates_per_second.value,
        &MIN_UPS,
        &MAX_UPS,
    );
    *options_state.options.updates_per_second = max(
        options_state.options.updates_per_second.value,
        options_state.options.frames_per_second.value,
    );
    *options_state.options.sound_channels = clamp(
        &options_state.options.sound_channels.value,
        &MIN_SOUND_CHANNELS,
        &MAX_SOUND_CHANNELS,
    );
    *options_state.options.sound_volume = clamp(
        &options_state.options.sound_volume.value,
        &MIN_VOLUME,
        &MAX_VOLUME,
    );
    *options_state.options.music_volume = clamp(
        &options_state.options.music_volume.value,
        &MIN_VOLUME,
        &MAX_VOLUME,
    );
    translations::set_current_language(&options_state.options.language.value);
    // TODO: Implement meee
    // crate::main::UpdateFrameRate();
}
