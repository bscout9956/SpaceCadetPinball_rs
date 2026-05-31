use crate::translations::Msg;
use sdl2::sys::SDL_MessageBoxFlags;
use std::cell::RefCell;
use std::ffi::c_char;
use std::sync::Weak;
use std::sync::atomic::{AtomicBool, AtomicUsize};

pub static QUICK_FLAG: AtomicBool = AtomicBool::new(false);
pub static FULL_TILT_MODE: AtomicBool = AtomicBool::new(false);

pub static FULL_TILT_DEMO_MODE: AtomicBool = AtomicBool::new(false);

pub static CHEAT_MODE: AtomicBool = AtomicBool::new(false);

pub static DEMO_MODE: AtomicBool = AtomicBool::new(false);

pub static CREDITS_ACTIVE: AtomicBool = AtomicBool::new(false);

pub static TIME_TICKS: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn make_path_name(p0: &str) -> String {
    todo!()
}

pub(crate) fn get_rc_string(p0: Msg) -> *const c_char {
    todo!()
}

pub(crate) fn show_message_box(p0: SDL_MessageBoxFlags, p1: &str, p2: *const c_char) {
    todo!()
}

pub(crate) fn select_dat_file(p0: Vec<&str>) {
    todo!()
}