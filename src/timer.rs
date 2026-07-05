use crate::state::pinball_state::PinballState;
use crate::utils::DrawContext;
use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{LazyLock, Mutex};
use thiserror::Error;

pub type TimerCallback = unsafe extern "C" fn(i32, *mut c_void, &mut DrawContext);

#[derive(Clone, Default)]
pub struct Timer {
    pub target_time: i32,
    pub caller: *mut c_void,
    pub callback: Option<TimerCallback>,
    pub next: i32,
    pub id: i32,
}

unsafe impl Sync for Timer {}
unsafe impl Send for Timer {}

const NONE: i32 = -1;

static TIMER_BUFFER: LazyLock<Mutex<Vec<Timer>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static ACTIVE_HEAD: AtomicI32 = AtomicI32::new(NONE);
static FREE_HEAD: AtomicI32 = AtomicI32::new(NONE);
static ACTIVE_COUNT: AtomicI32 = AtomicI32::new(0);
static CAPACITY: AtomicI32 = AtomicI32::new(0);
static NEXT_ID: AtomicI32 = AtomicI32::new(1);

#[derive(Error, Debug)]
pub enum TimerError {
    #[error("Failed to lock timer buffer")]
    LockPoisoned,
}

pub fn init(count: i32) -> Result<(), TimerError> {
    let mut data_buffer: Vec<Timer>;

    data_buffer = (0..count).map(|_| Timer::default()).collect();

    data_buffer.iter().for_each(|item| {
        (*TIMER_BUFFER.lock().unwrap()).push(item.clone());
    });

    for index in 0..(count - 1) {
        data_buffer[index as usize].next = index + 1;
    }
    data_buffer[(count - 1) as usize].next = NONE;

    FREE_HEAD.store(0, Relaxed);
    ACTIVE_HEAD.store(NONE, Relaxed);
    ACTIVE_COUNT.store(0, Relaxed);
    CAPACITY.store(count, Relaxed);
    NEXT_ID.store(count, Relaxed);

    Ok(())
}

pub fn uninit() {
    let mut buffer = TIMER_BUFFER.lock().unwrap();
    buffer.clear();
    FREE_HEAD.store(NONE, Relaxed);
    ACTIVE_HEAD.store(NONE, Relaxed);
    ACTIVE_COUNT.store(0, Relaxed);
    CAPACITY.store(0, Relaxed);
}

// fn get_time_ticks(time_ticks: usize) -> usize {
//     pb::TIME_TICKS.load(Relaxed)
// }

pub fn set(
    time: f32,
    caller: *mut c_void,
    callback: unsafe extern "C" fn(i32, *mut c_void, &mut DrawContext),
    draw_context: &mut DrawContext,
) -> i32 {
    let active_count = ACTIVE_COUNT.load(Relaxed);
    let capacity = CAPACITY.load(Relaxed);

    if active_count >= capacity {
        return 0;
    }

    let mut buffer = TIMER_BUFFER.lock().unwrap();

    let timer_idx = FREE_HEAD.load(Relaxed);
    if timer_idx == NONE {
        return 0;
    }

    let next_free = buffer[timer_idx as usize].next;
    FREE_HEAD.store(next_free, Relaxed);

    let target_time = (draw_context.time_ticks + (time * 1000.0) as usize) as i32;

    let mut prev = NONE;
    let mut current = ACTIVE_HEAD.load(Relaxed);
    let mut index = 0;

    while current != NONE
        && index < active_count
        && target_time >= buffer[current as usize].target_time
    {
        prev = current;
        current = buffer[current as usize].next;
        index += 1;
    }

    if prev != NONE {
        buffer[timer_idx as usize].next = buffer[prev as usize].next;
        buffer[prev as usize].next = timer_idx;
    } else {
        buffer[timer_idx as usize].next = ACTIVE_HEAD.load(Relaxed);
        ACTIVE_HEAD.store(timer_idx, Relaxed);
    }

    buffer[timer_idx as usize].caller = caller;
    buffer[timer_idx as usize].callback = Some(callback);
    buffer[timer_idx as usize].target_time = target_time;

    let timer_id = NEXT_ID.load(Relaxed);
    buffer[timer_idx as usize].id = timer_id;

    ACTIVE_COUNT.store(active_count + 1, Relaxed);

    NEXT_ID.fetch_add(1, Relaxed);

    timer_id
}

pub fn kill_callback(callback: unsafe extern "C" fn(i32, *mut c_void, &mut DrawContext)) -> i32 {
    let mut buffer = TIMER_BUFFER.lock().unwrap();
    let mut count = ACTIVE_COUNT.load(Relaxed);
    let mut kill_count = 0;

    let mut current = ACTIVE_HEAD.load(Relaxed);
    let mut prev = NONE;

    while current != NONE {
        let current_usize = current as usize;

        if buffer[current_usize].callback == Some(callback) {
            kill_count += 1;
            let next = buffer[current_usize].next;

            if prev != NONE {
                buffer[prev as usize].next = next;
            } else {
                ACTIVE_HEAD.store(next, Relaxed);
            }

            buffer[current_usize].next = FREE_HEAD.load(Relaxed);
            FREE_HEAD.store(current, Relaxed);

            count -= 1;

            current = next;
        } else {
            prev = current;
            current = buffer[current_usize].next;
        }
    }

    ACTIVE_COUNT.store(count, Relaxed);
    kill_count
}

pub fn kill_id(timer_id: i32) -> i32 {
    let mut buffer = TIMER_BUFFER.lock().unwrap();
    let mut count = ACTIVE_COUNT.load(Relaxed);

    if count <= 0 {
        return 0;
    }

    let mut current = ACTIVE_HEAD.load(Relaxed);
    let mut prev = NONE;
    let mut index = 0;

    while current != NONE {
        if buffer[current as usize].id == timer_id {
            break;
        }

        index += 1;
        prev = current;
        current = buffer[current as usize].next;

        if index >= count {
            return 0;
        }
    }

    if prev != NONE {
        buffer[prev as usize].next = buffer[current as usize].next;
    } else {
        ACTIVE_HEAD.store(buffer[current as usize].next, Relaxed);
    }

    buffer[current as usize].next = FREE_HEAD.load(Relaxed);
    FREE_HEAD.store(current, Relaxed);

    count -= 1;
    ACTIVE_COUNT.store(count, Relaxed);

    timer_id
}

pub fn check(time_ticks: usize, state: &mut PinballState) -> i32 {
    let mut buffer = TIMER_BUFFER.lock().unwrap();
    let mut count = ACTIVE_COUNT.load(Relaxed);
    let mut index = 0;

    let mut current = ACTIVE_HEAD.load(Relaxed);

    if current != NONE {
        while current != NONE && time_ticks as i32 >= buffer[current as usize].target_time {
            count -= 1;

            ACTIVE_HEAD.store(buffer[current as usize].next, Relaxed);

            buffer[current as usize].next = FREE_HEAD.load(Relaxed);
            FREE_HEAD.store(current, Relaxed);

            if let Some(callback) = buffer[current as usize].callback {
                let timer_id = buffer[current as usize].id;
                let caller = buffer[current as usize].caller;
                let mut draw_ctx = DrawContext {
                    v_screen: &mut state.render_state.v_screen,
                    current_palette: &state.pb_game_state.current_palette,
                    time_ticks: state.pb_game_state.time_ticks,
                    full_tilt_mode: state.pb_game_state.full_tilt_mode,
                    background_bitmap: &state.render_state.background_bitmap,
                };
                unsafe { callback(timer_id, caller, &mut draw_ctx) };
            }

            current = ACTIVE_HEAD.load(Relaxed);
            index += 1;

            if index > 1 {
                break;
            }

            if current == NONE {
                ACTIVE_COUNT.store(count, Relaxed);
                return index;
            }
        }

        while current != NONE && time_ticks as i32 >= buffer[current as usize].target_time + 100 {
            count -= 1;

            ACTIVE_HEAD.store(buffer[current as usize].next, Relaxed);

            buffer[current as usize].next = FREE_HEAD.load(Relaxed);
            FREE_HEAD.store(current, Relaxed);

            if let Some(callback) = buffer[current as usize].callback {
                let timer_id = buffer[current as usize].id;
                let caller = buffer[current as usize].caller;
                let mut draw_ctx = DrawContext {
                    v_screen: &mut state.render_state.v_screen,
                    current_palette: &state.pb_game_state.current_palette,
                    time_ticks: state.pb_game_state.time_ticks,
                    full_tilt_mode: state.pb_game_state.full_tilt_mode,
                    background_bitmap: &state.render_state.background_bitmap,
                };
                unsafe { callback(timer_id, caller, &mut draw_ctx) };
            }

            current = ACTIVE_HEAD.load(Relaxed);
            index += 1;
        }
    }

    ACTIVE_COUNT.store(count, Relaxed);
    index
}
