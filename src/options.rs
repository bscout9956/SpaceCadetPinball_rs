use crate::errors::FullscreenError;
use crate::options::InputTypes::{GameController, Keyboard, Mouse};
use crate::state::fullscrn_state::FullscrnState;
use crate::state::main_state::MainState;
use crate::state::options_state::OptionsState;
use crate::state::pb_game_state::PbGameState;
use crate::state::pinball_state::PinballState;
use crate::translations::Msg;
use crate::utils::clamp;
use crate::{fullscrn, midi, render, sound, translations, update_frame_rate};
use anyhow::{Context, Result};
use dear_imgui_rs::Io;
use dear_imgui_rs::sys::{
    ImGuiContext, ImGuiSettingsHandler, ImGuiTextBuffer, ImGuiTextBuffer_append,
    ImGuiTextBuffer_appendf, igAddSettingsHandler, igGetCurrentContext, igImHashStr,
    igLoadIniSettingsFromDisk, igMarkIniSettingsDirty_Nil,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use sdl2::sys::SDL_GameControllerButton::*;
use sdl2::sys::*;
use std::cmp::{PartialEq, PartialOrd, max};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::{CStr, CString, c_char, c_void};
use std::ops::{Deref, DerefMut, Sub};
use thiserror::Error;

pub const MIX_MAX_VOLUME: i32 = 100; // TODO: Is it 100?

#[allow(clippy::enum_variant_names)]
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
        let prefix = match self.input_type {
            InputTypes::None => {
                return String::from("Unused");
            }
            Keyboard => "Keyboard\n".to_string(),
            Mouse => "Mouse\n".to_string(),
            GameController => "GameController\n".to_string(),
        };
        unsafe { prefix + &self.get_short_input_description() }
    }

    pub unsafe fn get_short_input_description(&self) -> String {
        let mouse_buttons: [&str; 6] = ["", "Left", "Middle", "Right", "X1", "X2"];

        let controller_buttons: [&str; 21] = [
            "A",
            "B",
            "X",
            "Y",
            "Back",
            "Guide",
            "Start",
            "LeftStick",
            "RightStick",
            "LeftShoulder",
            "RightShoulder",
            "DpUp",
            "DpDown",
            "DpLeft",
            "DpRight",
            "Misc1",
            "Paddle1",
            "Paddle2",
            "Paddle3",
            "Paddle4",
            "Touchpad",
        ];

        match self.input_type {
            InputTypes::None => "".to_string(),
            Keyboard => unsafe {
                let key_name = SDL_GetKeyName(self.value);
                CStr::from_ptr(key_name).to_string_lossy().into_owned()
            },
            Mouse => {
                if self.value >= SDL_BUTTON_LEFT as i32 && self.value <= SDL_BUTTON_X2 as i32 {
                    mouse_buttons[self.value as usize].to_string()
                } else {
                    format!("MButton {}", self.value)
                }
            }
            GameController => {
                if self.value >= SDL_CONTROLLER_BUTTON_A as i32
                    && self.value < i32::min(SDL_CONTROLLER_BUTTON_MAX as i32, 21)
                {
                    controller_buttons[self.value as usize].to_string()
                } else {
                    format!("CButton {}", self.value)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, FromPrimitive)]
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

pub fn get_int(name: &str, default_value: i32, settings: &mut HashMap<String, String>) -> i32 {
    let settings = get_setting(name, &default_value.to_string(), settings);
    settings.parse::<i32>().unwrap_or(default_value)
}

pub fn get_setting(
    key: &str,
    default_value: &str,
    settings: &mut HashMap<String, String>,
) -> String {
    match settings.entry(key.to_string()) {
        Entry::Occupied(entry) => entry.get().clone(),
        Entry::Vacant(entry) => {
            let new_value = entry.insert(default_value.to_string());
            unsafe {
                let ctx = igGetCurrentContext();
                if !ctx.is_null() {
                    igMarkIniSettingsDirty_Nil();
                }
            }
            new_value.clone()
        }
    }
}

pub fn set_int(name: &str, data: i32, settings: &mut HashMap<String, String>) {
    set_setting(name, &data.to_string(), settings);
}

fn set_setting(key: &str, value: &String, settings: &mut HashMap<String, String>) {
    settings.insert(key.to_string(), value.to_string());
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
    fn load(&mut self, settings: &mut HashMap<String, String>);
    fn save(&mut self, settings: &mut HashMap<String, String>);
    fn reset(&mut self);
}

pub trait SettingValue: Clone {
    fn fetch(name: &str, default: Self, settings: &mut HashMap<String, String>) -> Self;
    fn store(name: &str, value: &Self, settings: &mut HashMap<String, String>);
}

impl SettingValue for i32 {
    fn fetch(name: &str, default: i32, settings: &mut HashMap<String, String>) -> Self {
        match settings.get(name) {
            Some(value) => value.parse::<i32>().unwrap_or(default),
            Option::None => default,
        }
    }
    fn store(name: &str, value: &i32, settings: &mut HashMap<String, String>) {
        settings.insert(name.to_string(), value.to_string());
    }
}

impl SettingValue for bool {
    fn fetch(name: &str, default: Self, settings: &mut HashMap<String, String>) -> Self {
        match settings.get(name) {
            Some(value) => value.parse::<bool>().unwrap_or(default),
            Option::None => default,
        }
    }
    fn store(name: &str, value: &bool, settings: &mut HashMap<String, String>) {
        settings.insert(name.to_string(), value.to_string());
    }
}

impl SettingValue for String {
    fn fetch(name: &str, default: Self, settings: &mut HashMap<String, String>) -> Self {
        match settings.get(name) {
            Some(value) => value.to_string(),
            Option::None => default,
        }
    }

    fn store(name: &str, value: &String, settings: &mut HashMap<String, String>) {
        settings.insert(name.to_string(), value.to_string());
    }
}

impl SettingValue for f32 {
    fn fetch(name: &str, default: Self, settings: &mut HashMap<String, String>) -> Self {
        match settings.get(name) {
            Some(value) => value.parse::<f32>().unwrap_or(default),
            Option::None => default,
        }
    }

    fn store(name: &str, value: &f32, settings: &mut HashMap<String, String>) {
        settings.insert(name.to_string(), value.to_string());
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
    fn load(&mut self, settings: &mut HashMap<String, String>) {
        self.value = T::fetch(self.name, self.default.clone(), settings);
    }

    fn save(&mut self, settings: &mut HashMap<String, String>) {
        T::store(self.name, &self.value, settings);
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
    pub name: &'static str,
    pub description: Msg,
    pub defaults: [GameInput; 3],
    pub inputs: [GameInput; 3],
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

    pub fn get_shortcut_description(&self) -> String {
        let result: String = String::new();
        for input in self.inputs {
            if input.input_type != InputTypes::None {
                return unsafe { input.get_short_input_description() };
            }
        }
        result
    }
}

impl OptionBase for ControlOption {
    fn load(&mut self, settings: &mut HashMap<String, String>) {
        for (idx, input) in self.inputs.iter_mut().enumerate() {
            let name = format!("{} {}", self.name, idx);
            input.input_type = self.defaults[idx].input_type;
            input.value = get_int(
                &format!("{} input", name),
                self.defaults[idx].value,
                settings,
            );
        }
    }

    fn save(&mut self, settings: &mut HashMap<String, String>) {
        for (idx, input) in self.inputs.iter_mut().enumerate() {
            let name = String::from(self.name) + " " + &idx.to_string();
            set_int(&format!("{} type", name), input.input_type as i32, settings);
            set_int(&format!("{} input", name), input.value, settings);
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

    pub fn load_all(&mut self, settings: &mut HashMap<String, String>) {
        for opt in self.all_options_mut() {
            opt.load(settings);
        }
    }

    pub fn save_all(&mut self, settings: &mut HashMap<String, String>) {
        for opt in self.all_options_mut() {
            opt.save(settings);
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
pub unsafe fn init_primary(
    io: &mut Io,
    main_state: &mut MainState,
    options_state: &mut OptionsState,
) {
    unsafe {
        let im_context = igGetCurrentContext();
        let mut ini_handler: ImGuiSettingsHandler = std::mem::zeroed();

        ini_handler.TypeName = c"Pinball".as_ptr();
        ini_handler.TypeHash = igImHashStr(ini_handler.TypeName, 0, 0);

        ini_handler.UserData =
            &mut options_state.settings as *mut HashMap<String, String> as *mut c_void;

        ini_handler.ReadOpenFn = Some(MyUserData_ReadOpen);
        ini_handler.ReadLineFn = Some(MyUserData_ReadLine);
        ini_handler.WriteAllFn = Some(MyUserData_WriteAll);

        igAddSettingsHandler(&ini_handler);

        if !(*im_context).SettingsLoaded {
            igLoadIniSettingsFromDisk((*im_context).IO.IniFilename);
            (*im_context).SettingsLoaded = true;
        }

        options_state.options.load_all(&mut options_state.settings);
        post_process_options(io, main_state, options_state);
    }
}

#[derive(Error, Debug)]
pub enum OptionsError {
    #[error(transparent)]
    FullScreenError(#[from] FullscreenError),
}

pub fn init_secondary(
    options_state: &mut OptionsState,
    pb_game_state: &mut PbGameState,
    fullscrn_state: &mut FullscrnState,
) -> Result<(), OptionsError> {
    let max_res = fullscrn::get_max_resolution(pb_game_state);

    if options_state.options.resolution.value >= 0
        && options_state.options.resolution.value > max_res
    {
        *options_state.options.resolution = max_res;
    }
    if options_state.options.resolution.value == -1 {
        fullscrn::set_resolution(max_res, fullscrn_state, pb_game_state)?;
    } else {
        fullscrn::set_resolution(
            *options_state.options.resolution,
            fullscrn_state,
            pb_game_state,
        )?;
    }
    Ok(())
}

pub fn uninit(options_state: &mut OptionsState) {
    if let Some(cur_lang) = translations::get_current_language() {
        options_state.options.language.value = cur_lang.short_name.to_string();
        options_state.options.save_all(&mut options_state.settings);
    } else {
        println!("Unable to obtain current language info...");
    }
}

pub fn get_input(
    row_name: &str,
    values: &mut [GameInput; 3],
    settings: &mut HashMap<String, String>,
) {
    for (index, input) in values.iter_mut().enumerate() {
        let name = format!("{} {}", row_name, index);
        let type_val = get_int(&format!("{} type", name), -1, settings);
        let input_type: InputTypes = InputTypes::from_i32(type_val).unwrap();
        let value = get_int(&format!("{} input", name), -1, settings);

        if input_type <= GameController && value != -1 {
            *input = GameInput { input_type, value };
        }
    }
}

pub fn set_input(
    row_name: &str,
    values: &mut [GameInput; 3],
    settings: &mut HashMap<String, String>,
) {
    for (index, input) in values.iter_mut().enumerate() {
        let name = format!("{} {}", row_name, index);
        set_int(&format!("{} type", name), input.input_type as i32, settings);
        set_int(&format!("{} input", name), input.value, settings);
    }
}

impl Sub for Menu {
    type Output = i32;

    fn sub(self, rhs: Self) -> i32 {
        self as i32 - rhs as i32
    }
}

pub fn toggle(u_id_check_item: Menu, state: &mut PinballState) -> Result<()> {
    match u_id_check_item {
        Menu::NewGame => Ok(()),
        Menu::AboutPinball => Ok(()),
        Menu::HighScores => Ok(()),
        Menu::Exit => Ok(()),
        Menu::Sounds => {
            *state.options_state.options.sounds ^= true;
            sound::enable(*state.options_state.options.sounds, &mut state.sound_state);
            Ok(())
        }
        Menu::Music => {
            *state.options_state.options.music ^= true;
            if !(*state.options_state.options.music) {
                midi::music_stop();
            } else {
                midi::music_play();
            }
            Ok(())
        }
        Menu::SoundStereo => {
            *state.options_state.options.sound_stereo ^= true;
            Ok(())
        }
        Menu::FullScreen => {
            *state.options_state.options.full_screen ^= true;
            fullscrn::set_screen_mode(
                *state.options_state.options.full_screen,
                &mut state.fullscrn_state,
                &mut state.main_state.main_window,
            )
            .context("Failed to set screen mode")?;
            Ok(())
        }
        Menu::Demo => Ok(()),
        Menu::SelectTable => Ok(()),
        Menu::PlayerControls => Ok(()),
        Menu::OnePlayer | Menu::TwoPlayers | Menu::ThreePlayers | Menu::FourPlayers => {
            *state.options_state.options.players = u_id_check_item - Menu::OnePlayer + 1;
            Ok(())
        }
        Menu::ShowMenu => {
            *state.options_state.options.show_menu = !(*state.options_state.options.show_menu);
            fullscrn::window_size_changed(state).context("Failed to change window size")?;
            Ok(())
        }
        Menu::MaximumResolution | Menu::R640x480 | Menu::R800x600 | Menu::R1024x768 => {
            let mut restart = false;
            let new_resolution = u_id_check_item as i32 - Menu::R640x480 as i32;
            if u_id_check_item == Menu::MaximumResolution {
                restart = state.fullscrn_state.resolution
                    != fullscrn::get_max_resolution(&mut state.pb_game_state);
                *state.options_state.options.resolution = -1;
            } else if new_resolution <= fullscrn::get_max_resolution(&mut state.pb_game_state) {
                let resolution = if *state.options_state.options.resolution == -1 {
                    fullscrn::get_max_resolution(&mut state.pb_game_state)
                } else {
                    state.fullscrn_state.resolution
                };
                if new_resolution != resolution {
                    restart = new_resolution != resolution;
                }
                *state.options_state.options.resolution = new_resolution;
            }

            if restart {
                crate::restart(&mut state.main_state);
            }
            Ok(())
        }
        Menu::WindowUniformScale => {
            *state.options_state.options.uniform_scaling ^= true;
            fullscrn::window_size_changed(state).context("Failed to change window size")?;
            Ok(())
        }
        Menu::WindowLinearFilter => {
            *state.options_state.options.linear_filtering ^= true;
            render::recreate_screen_texture(
                &mut state.main_state,
                &mut state.options_state,
                &mut state.render_state,
            );
            Ok(())
        }
        Menu::WindowIntegerScale => {
            *state.options_state.options.integer_scaling ^= true;
            fullscrn::window_size_changed(state).context("Failed to change window size")?;
            Ok(())
        }
        Menu::Prefer3DPBGameData => {
            *state.options_state.options.prefer_3dpb_game_data ^= true;
            fullscrn::window_size_changed(state).context("Failed to change window size")?;
            Ok(())
        }
        _ => Ok(()),
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

pub fn render_control_dialog(show_dialog: bool) {
    if !show_dialog {
        return;
    }

    // TODO: ImGui stuff

    if !show_dialog {}
}

pub fn map_game_input(key: GameInput, options_state: &mut OptionsState) -> Vec<GameBindings> {
    let mut result: Vec<GameBindings> = Vec::new();

    for input_id in GameBindings::Min as i32..GameBindings::Exit as i32 {
        for input_value in options_state.options.control_options[input_id as usize].inputs {
            if key == input_value {
                // Must add 1 because of the Min
                result.push(GameBindings::from_i32(input_id + 1).unwrap());
                break;
            }
        }
    }

    result
}

pub fn reset_all_options(
    io: &mut Io,
    main_state: &mut MainState,
    options_state: &mut OptionsState,
) {
    options_state.options.reset_all();
    post_process_options(io, main_state, options_state);
}

#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn MyUserData_ReadLine(
    ctx: *mut ImGuiContext,
    handler: *mut ImGuiSettingsHandler,
    entry: *mut c_void,
    line: *const c_char,
) {
    if entry.is_null() {
        return;
    }

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

#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn MyUserData_ReadOpen(
    ctx: *mut ImGuiContext,
    handler: *mut ImGuiSettingsHandler,
    name: *const c_char,
) -> *mut c_void {
    unsafe {
        let name_str = CStr::from_ptr(name);
        if name_str.to_bytes() == b"Settings" {
            return (*handler).UserData;
        }
        std::ptr::null_mut()
    }
}

#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn MyUserData_WriteAll(
    ctx: *mut ImGuiContext,
    handler: *mut ImGuiSettingsHandler,
    buf: *mut ImGuiTextBuffer,
) {
    unsafe {
        if (*handler).UserData.is_null() {
            return;
        }

        let settings = &*((*handler).UserData as *mut HashMap<String, String>);

        ImGuiTextBuffer_appendf(
            buf,
            c"[%s][%s]\n".as_ptr(),
            (*handler).TypeName,
            c"Settings".as_ptr(),
        );

        for (key, value) in settings.iter() {
            if let Ok(line) = CString::new(format!("{}={}\n", key, value)) {
                ImGuiTextBuffer_append(buf, line.as_ptr(), std::ptr::null());
            }
        }

        ImGuiTextBuffer_append(buf, c"\n".as_ptr(), std::ptr::null());
    }
}

pub fn post_process_options(
    io: &mut Io,
    main_state: &mut MainState,
    options_state: &mut OptionsState,
) {
    io.set_font_global_scale(*options_state.options.ui_scale);
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
    update_frame_rate(main_state, options_state);
}

pub(crate) fn input_down(input: GameInput, options_state: &mut OptionsState) {
    if options_state.control_waiting_for_input.is_some() {
        options_state.control_waiting_for_input = Some(input);
        options_state.control_waiting_for_input = Option::None;
    }
}

pub(crate) fn show_control_dialog(options_state: &mut OptionsState) {
    if !options_state.show_dialog {
        options_state.control_waiting_for_input = Option::None;
        options_state.show_dialog = true;

        for ctrl in options_state.options.control_options.iter_mut() {
            ctrl.save(&mut options_state.settings);
        }
    }
}
