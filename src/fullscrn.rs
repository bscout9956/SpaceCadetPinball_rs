use crate::options::Setting;
use crate::pb;
use std::sync::atomic;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::SeqCst;

static RESOLUTION: AtomicI32 = AtomicI32::new(0);

pub fn set_resolution(mut value: i32) {
    if pb::FULL_TILT_MODE.load(SeqCst) && !pb::FULL_TILT_DEMO_MODE.load(SeqCst) {
        value = 0;
    }
    assert!(value >= 0 && value <= 2, "Resolution value out of bounds");
    RESOLUTION.store(value, SeqCst);
}

pub fn get_max_resolution() -> i32 {
    if pb::FULL_TILT_MODE.load(SeqCst) && !pb::FULL_TILT_DEMO_MODE.load(SeqCst) {
        2
    } else {
        0
    }
}

pub fn set_screen_mode(p0: bool) {
    todo!()
}

pub fn get_resolution() -> i32 {
    RESOLUTION.load(atomic::Ordering::Acquire)
}

pub fn window_size_changed() {
    todo!()
}
