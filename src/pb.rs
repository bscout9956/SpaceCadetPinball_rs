use std::sync::atomic::AtomicBool;

pub(crate) static QUICK_FLAG: AtomicBool = AtomicBool::new(false);
pub static FULL_TILT_MODE: AtomicBool = AtomicBool::new(false);

pub static FULL_TILT_DEMO_MODE: AtomicBool = AtomicBool::new(false);

pub static CHEAT_MODE: AtomicBool = AtomicBool::new(false);

pub static DEMO_MODE: AtomicBool = AtomicBool::new(false);

pub static CREDITS_ACTIVE: AtomicBool = AtomicBool::new(false);

pub(crate) fn make_path_name(p0: &str) -> String {
    todo!()
}

pub(crate) fn ShowMessageBox(p0: _, p1: &str, p2: &str) {
    todo!()
}
