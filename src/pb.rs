use crate::errors::PbInitError;
use crate::group_data::DatFile;
use crate::options::OPTIONS;
use crate::translations::{Msg, TranslationError};
use crate::{partman, score, translations};
use sdl2::sys::SDL_MessageBoxFlags;
use std::ffi::c_char;
use std::fs::File;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{LazyLock, Mutex};

pub static QUICK_FLAG: AtomicBool = AtomicBool::new(false);
pub static FULL_TILT_MODE: AtomicBool = AtomicBool::new(false);

pub static FULL_TILT_DEMO_MODE: AtomicBool = AtomicBool::new(false);

pub static CHEAT_MODE: AtomicBool = AtomicBool::new(false);

pub static DEMO_MODE: AtomicBool = AtomicBool::new(false);

pub static CREDITS_ACTIVE: AtomicBool = AtomicBool::new(false);

pub static TIME_TICKS: AtomicUsize = AtomicUsize::new(0);

pub static DAT_FILE_NAME: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static BASE_PATH: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static RECORD_TABLE: LazyLock<Mutex<Option<DatFile>>> = LazyLock::new(|| Mutex::new(None));

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

pub fn init() -> Result<(bool), PbInitError> {
    let projection_matrix: [f32; 12] = [0.0; 12];

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

            let use_bmp_font: i32 = 0;
            get_rc_int(Msg::TextBoxUseBitmapFont, &use_bmp_font);
            if use_bmp_font == 1 {
                score::load_msg_font("pbmsg_ft");
            }

            if record_table.is_none() {
                return Ok(true);
            }
            
            
        }
        Err(e) => {
            println!("Error locking RECORD_TABLE: {}", e);
        }
    }

    let use_bmp_font = false;

    Ok(false)
}

// Note: This used to be code that took a string like "1 Blablabla" and would get only the first part of the string
// It's an old CPP programming practice apparently,
// I guess valued enums weren't a thing? I am not quite sure
pub fn get_rc_int(u_id: Msg, use_bmp_font: &i32) -> Result<i32, TranslationError> {
    let s = get_rc_string(u_id)?;

    let first_char = s.split_whitespace().next().unwrap_or("0");

    Ok(first_char.parse::<i32>().unwrap_or(0))
}
