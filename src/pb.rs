use crate::errors::PbError;
use crate::fullscrn::RESOLUTION_ARRAY;
use crate::gdrv::{ColorRgba, GdrvBitmap8};
use crate::group_data::{DatFile, EntryBuffer, FieldTypes};
use crate::options::OPTIONS;
use crate::t_pinball_component::MessageCode;
use crate::t_pinball_table::TPinballTable;
use crate::t_textbox::TTextBox;
use crate::translations::{Msg, TranslationError};
use crate::{
    DEMO_ACTIVE, HIGH_SCORES_ENABLED, LAUNCH_BALL_ENABLED, fullscrn, gdrv, loader, partman, proj,
    render, score, timer, translations,
};
use sdl2::sys::SDL_MessageBoxFlags;
use std::cell::RefCell;
use std::ffi::c_char;
use std::fs::File;
use std::rc::Rc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{LazyLock, Mutex};

pub static QUICK_FLAG: AtomicBool = AtomicBool::new(false);
pub static FULL_TILT_MODE: AtomicBool = AtomicBool::new(false);

pub static FULL_TILT_DEMO_MODE: AtomicBool = AtomicBool::new(false);

pub static CHEAT_MODE: AtomicBool = AtomicBool::new(false);

pub static DEMO_MODE: AtomicBool = AtomicBool::new(false);

pub static CREDITS_ACTIVE: AtomicBool = AtomicBool::new(false);

pub static IDLE_TIMER_MS: Mutex<f32> = Mutex::new(0.0);

pub static TIME_TICKS: AtomicUsize = AtomicUsize::new(0);

pub static DAT_FILE_NAME: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static BASE_PATH: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static RECORD_TABLE: LazyLock<Mutex<Option<DatFile>>> = LazyLock::new(|| Mutex::new(None));

pub static MAIN_TABLE: LazyLock<Mutex<Option<TPinballTable>>> = LazyLock::new(|| Mutex::new(None));

pub static MISS_TEXT_BOX: Mutex<Option<TTextBox>> = Mutex::new(None);

pub static GAME_MODE: Mutex<GameModes> = Mutex::new(GameModes::GameOver);

#[derive(PartialEq, Eq, Ord, PartialOrd)]
pub enum GameModes {
    InGame = 1,
    GameOver = 2,
}

pub fn make_path_name(file_name: &str) -> String {
    match BASE_PATH.lock() {
        Ok(path) => {
            return format!("{}{}", *path, file_name);
        }
        Err(e) => {
            println!("Failed to lock base_path {}", e);
        }
    }
    String::new()
}

pub fn get_rc_string(u_id: Msg) -> Result<&'static str, TranslationError> {
    translations::get_translation(u_id)
}

pub fn show_message_box(p0: SDL_MessageBoxFlags, p1: &str, p2: *const c_char) {
    todo!()
}

pub fn select_dat_file(data_search_paths: &[&str]) {
    clear_dat_file_name();
    FULL_TILT_MODE.store(false, Relaxed);
    FULL_TILT_DEMO_MODE.store(false, Relaxed);

    let mut dat_file_names: [&str; 3] = ["CADET.DAT", "PINBALL.DAT", "DEMO.DAT"];

    match OPTIONS.lock() {
        Ok(mut options) => {
            if options.prefer_3dpb_game_data.value {
                dat_file_names.swap(0, 1);
            }
        }
        Err(e) => {
            println!("Error locking OPTIONS: {}", e);
        }
    }

    for path in data_search_paths {
        if path.is_empty() {
            continue;
        }

        set_base_path(path);

        for dat_file_name in dat_file_names {
            let mut file_name = dat_file_name.to_string();
            for i in 0..2 {
                if i == 1 {
                    file_name = file_name.to_lowercase();
                }

                let dat_file_path = make_path_name(&file_name);
                if let Err(e) = File::open(&dat_file_path) {
                    println!("Error opening dat_file {}: {}", &dat_file_path, e);
                    continue;
                }
                set_dat_file_name(&file_name);

                update_full_tilt_mode(dat_file_name);
                return;
            }
        }
    }
}

fn clear_dat_file_name() {
    match DAT_FILE_NAME.lock() {
        Ok(mut file_name) => {
            file_name.clear();
        }
        Err(e) => {
            println!("Error locking DAT_FILE_NAME: {}", e);
        }
    }
}

fn set_dat_file_name(file_name: &str) {
    match DAT_FILE_NAME.lock() {
        Ok(mut dat_name) => {
            *dat_name = String::from(file_name);
        }
        Err(e) => {
            println!("Error locking DAT_FILE_NAME: {}", e);
        }
    }
}

fn update_full_tilt_mode(dat_file_name: &str) {
    if dat_file_name == "CADET.DAT" {
        FULL_TILT_MODE.store(true, Relaxed);
    }
    if dat_file_name == "DEMO.DAT" {
        FULL_TILT_MODE.store(true, Relaxed);
        FULL_TILT_DEMO_MODE.store(true, Relaxed);
    }
}

fn set_base_path(path: &str) {
    match BASE_PATH.lock() {
        Ok(mut base_path) => {
            *base_path = String::from(path);
        }
        Err(e) => {
            println!("Error locking BASE_PATH: {}", e);
        }
    }
}

fn read_camera_floats(float_data: &[u8]) -> Vec<f32> {
    let mut data: Vec<f32> = Vec::new();

    for i in 0..12 {
        let offset = i * 4;
        data.push(f32::from_le_bytes([
            float_data[offset],
            float_data[offset + 1],
            float_data[offset + 2],
            float_data[offset + 3],
        ]));
    }

    data
}

pub fn init() -> Result<(bool), PbError> {
    let mut projection_matrix: [f32; 12] = [0.0; 12];

    let mut data_file_path = String::new();

    match DAT_FILE_NAME.lock() {
        Ok(mut file_name) => {
            if file_name.is_empty() {
                return Ok(false);
            }
            data_file_path = make_path_name(&file_name);
        }
        Err(e) => {
            println!("Error locking DAT_FILE_NAME: {}", e);
            return Ok(false);
        }
    }

    match RECORD_TABLE.lock() {
        Ok(mut record_table) => {
            *record_table = Some(partman::load_records(
                data_file_path,
                FULL_TILT_MODE.load(Relaxed),
            )?);
        }
        Err(e) => {
            println!("Error locking RECORD_TABLE: {}", e);
        }
    }

    let use_bmp_font: i32 = get_rc_int(Msg::TextBoxUseBitmapFont)?;
    if use_bmp_font == 1 {
        score::load_msg_font("pbmsg_ft");
    }

    match RECORD_TABLE.lock() {
        Ok(mut record_table) => {
            if record_table.is_none() {
                return Ok(true);
            } else {
                let table = record_table.as_mut().unwrap();
                let plt = table.field_labeled("background", FieldTypes::Palette);
                let plt_data = plt.unwrap();
                match plt_data {
                    EntryBuffer::Raw(data) => {
                        let mut palette_colors = Vec::with_capacity(256);
                        // extract method here
                        for i in 0..256 {
                            let offset = i * 4;
                            if offset + 3 < data.len() {
                                let color = u32::from_le_bytes([
                                    data[offset],
                                    data[offset + 1],
                                    data[offset + 2],
                                    data[offset + 3],
                                ]);
                                palette_colors.push(ColorRgba::color_rgba_u32(color));
                            } else {
                                palette_colors.push(ColorRgba::black());
                            }
                        }

                        gdrv::display_palette(Some(&palette_colors));
                    }
                    _ => {}
                }

                let mut background_bmp = table
                    .get_bitmap(table.record_labeled("background"))
                    .to_owned();
                let camera_info_id =
                    table.record_labeled("camera_info") + fullscrn::get_resolution();
                let camera_data = table.field(camera_info_id, FieldTypes::FloatArray).unwrap();
                let mut camera_info: Vec<f32> = Vec::new();
                match camera_data {
                    EntryBuffer::Raw(float_data) => {
                        camera_info = read_camera_floats(float_data);
                    }
                    _ => {}
                }
                let res_array = RESOLUTION_ARRAY.lock().unwrap();
                let res_info = &(*res_array)[fullscrn::get_resolution() as usize];

                if !camera_info.is_empty() {
                    projection_matrix.copy_from_slice(&camera_info);

                    let proj_center_x = res_info.table_width as f32 * 0.5;
                    let proj_center_y = res_info.table_height as f32 * 0.5;
                    let proj_d = camera_info[0];
                    let z_min = camera_info[1];
                    let z_scaler = camera_info[2];
                    proj::init(
                        projection_matrix,
                        proj_d,
                        proj_center_x,
                        proj_center_y,
                        z_min,
                        z_scaler,
                    );
                }

                render::init(None, res_info.table_width, res_info.table_height);

                let mut v_guard = render::V_SCREEN.lock().unwrap();
                if let Some(ref mut dst) = *v_guard {
                    gdrv::copy_bitmap(
                        dst,
                        background_bmp.width,
                        background_bmp.height,
                        background_bmp.x_position,
                        background_bmp.y_position,
                        background_bmp,
                        0,
                        0,
                    );
                }

                loader::load_from(table)?;
            }
        }
        Err(e) => {
            println!("Error locking RECORD_TABLE {}", e);
        }
    }

    // TODO: Implement modechange and gamemodes
    //mode_change(GameModes::InGame);

    TIME_TICKS.store(0, Relaxed);
    // TODO: Implement timer init
    //timer::init(150);
    // TODO: Implement score init
    //score::init();

    Ok(true)
}

// Note: This used to be code that took a string like "1 Blablabla" and would get only the first part of the string
// It's an old CPP programming practice apparently,
// I guess valued enums weren't a thing? I am not quite sure
pub fn get_rc_int(u_id: Msg) -> Result<i32, TranslationError> {
    let s = get_rc_string(u_id)?;

    let first_char = s.split_whitespace().next().unwrap_or("0");

    Ok(first_char.parse::<i32>().unwrap_or(0))
}

pub fn reset_table() -> Result<(), PbError> {
    let mut table_opt = MAIN_TABLE.lock()?;
    match table_opt.as_mut() {
        Some(mut main_table) => {
            main_table.message(MessageCode::RESET, 0.0);
            Ok(())
        }
        None => Ok(()),
    }
}

pub fn first_time_setup() {
    render::update();
}

pub(crate) fn toggle_demo() {
    todo!()
}

pub(crate) fn replay_level(demo_mode: bool) -> Result<(), PbError> {
    DEMO_MODE.store(demo_mode, Relaxed);
    mode_change(GameModes::InGame)?;
    Ok(())
}

fn mode_change(mode: GameModes) -> Result<(), PbError> {
    let mut credits_active = CREDITS_ACTIVE.load(Relaxed);
    let box_guard = MISS_TEXT_BOX.lock().map_err(|_| PbError::LockGeneric)?;
    let miss_text_box = (*box_guard);
    let mut idle_guard = IDLE_TIMER_MS.lock().map_err(|_| PbError::LockGeneric)?;

    if credits_active && miss_text_box.is_some() {
        miss_text_box.unwrap().clear(true);
    }
    credits_active = false;
    CREDITS_ACTIVE.store(credits_active, Relaxed);
    *idle_guard = 0.0;

    match mode {
        GameModes::InGame => {
            if (DEMO_MODE.load(Relaxed) == true) {
                LAUNCH_BALL_ENABLED.store(false, Relaxed);
                HIGH_SCORES_ENABLED.store(false, Relaxed);
                DEMO_ACTIVE.store(true, Relaxed);
                let mut main_table = MAIN_TABLE.lock()?;
                match main_table.as_mut() {
                    Some(table) => {
                        if table.demo.is_some() {
                            table.demo.as_mut().unwrap().active_flag = true;
                        }
                    }
                    None => {}
                }
            } else {
                LAUNCH_BALL_ENABLED.store(true, Relaxed);
                HIGH_SCORES_ENABLED.store(true, Relaxed);
                DEMO_ACTIVE.store(false, Relaxed);
                let mut main_table = MAIN_TABLE.lock()?;
                match main_table.as_mut() {
                    Some(mut table) => {
                        if table.demo.is_some() {
                            let table_demo = table.demo.as_mut().unwrap();
                            table_demo.active_flag = true;
                        }
                    }
                    None => {}
                }
            }
        }
        GameModes::GameOver => {
            LAUNCH_BALL_ENABLED.store(false, Relaxed);
            if DEMO_MODE.load(Relaxed) == false {
                HIGH_SCORES_ENABLED.store(true, Relaxed);
                DEMO_ACTIVE.store(false, Relaxed);
            }
            let main_table = MAIN_TABLE.lock()?;
            match (main_table.as_ref()) {
                Some(table) => {
                    if table.light_group.is_some() {
                        let light_group = table.light_group.as_ref().unwrap();
                        light_group.message(MessageCode::T_LIGHT_GROUP_GAME_OVER_ANIMATION, 1.4f32);
                    }
                }
                None => {}
            }
        }
    }
    let mut game_mode_grd = GAME_MODE.lock().map_err(|_| PbError::LockGeneric)?;
    *game_mode_grd = mode;

    Ok(())
}
