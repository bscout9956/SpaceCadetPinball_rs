use crate::group_data::DatFile;
use crate::options::OPTIONS;
use crate::partman;
use crate::translations::Msg;
use sdl2::sys::SDL_MessageBoxFlags;
use std::cell::RefCell;
use std::ffi::c_char;
use std::fs::File;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{LazyLock, LockResult, Mutex, Weak};

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

pub fn get_rc_string(p0: Msg) -> *const c_char {
    todo!()
}

pub fn show_message_box(p0: SDL_MessageBoxFlags, p1: &str, p2: *const c_char) {
    todo!()
}

pub fn select_dat_file(data_search_paths: &[&str]) {
    match DAT_FILE_NAME.lock() {
        Ok(mut file_name) => {
            file_name.clear();
        }
        Err(e) => {
            println!("Error locking DAT_FILE_NAME: {}", e);
        }
    }
    FULL_TILT_MODE.store(false, Relaxed);
    FULL_TILT_DEMO_MODE.store(false, Relaxed);

    let mut dat_file_names: [&str; 3] = ["CADET.DAT", "PINBALL.DAT", "DEMO.DAT"];

    match OPTIONS.lock() {
        Ok(mut options) => {
            if options.prefer_3dpb_game_data.value == true {
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

        match BASE_PATH.lock() {
            Ok(mut base_path) => {
                *base_path = String::from(*path);
            }
            Err(e) => {
                println!("Error locking BASE_PATH: {}", e);
            }
        }

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
                println!("Opened dat_file: {}", &dat_file_path);
                match DAT_FILE_NAME.lock() {
                    Ok(mut dat_file) => {
                        *dat_file = file_name;
                    }
                    Err(e) => {
                        println!("Error locking DAT_FILE_NAME: {}", e);
                    }
                }
                if dat_file_name == "CADET.DAT" {
                    FULL_TILT_MODE.store(true, Relaxed);
                }
                if dat_file_name == "DEMO.DAT" {
                    FULL_TILT_MODE.store(true, Relaxed);
                    FULL_TILT_DEMO_MODE.store(true, Relaxed);
                }
                println!("Loading game from: {}", &dat_file_path);
                return;
            }
        }
    }
}

pub fn init() -> bool {
    let projection_matrix: [f32; 12] = [0.0; 12];

    let mut data_file_path = String::new();
    match DAT_FILE_NAME.lock() {
        Ok(mut file_name) => {
            if file_name.is_empty() {
                return false;
            }
            data_file_path = make_path_name(&(*file_name));
        }
        Err(e) => {
            println!("Error locking DAT_FILE_NAME: {}", e);
            return false;
        }
    }

    match RECORD_TABLE.lock() {
        Ok(mut table_opt) => {
            *table_opt = partman::load_records(data_file_path, FULL_TILT_MODE.load(Relaxed));
        }
        Err(e) => {
            println!("Error locking RECORD_TABLE: {}", e);
        }
    }

    let use_bmp_font = false;

    false
}
