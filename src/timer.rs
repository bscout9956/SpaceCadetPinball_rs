use crate::state::pinball_state::PinballState;
use crate::t_textbox::TTextBox;
use crate::utils::DrawContext;
use std::ffi::c_void;
use std::ptr::null_mut;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{LazyLock, Mutex};
use thiserror::Error;

type CallBackFn = Box<dyn FnMut(i32, *mut c_void)>;

#[derive(Clone)]
pub struct TimerStruct {
    pub target_time: i32,
    pub caller: *mut c_void,
    pub callback: Option<unsafe extern "C" fn(i32, *mut c_void, &mut DrawContext)>,
    pub next_timer: i32,
    pub timer_id: i32,
}

unsafe impl Sync for TimerStruct {}
unsafe impl Send for TimerStruct {}

impl TimerStruct {
    pub fn new() -> Self {
        Self {
            target_time: 0,
            caller: null_mut(),
            callback: None,
            next_timer: -1,
            timer_id: 0,
        }
    }
}

impl Default for TimerStruct {
    fn default() -> Self {
        Self::new()
    }
}

static TIMER_BUFFER: LazyLock<Mutex<Vec<TimerStruct>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static ACTIVE_HEAD: AtomicI32 = AtomicI32::new(-1);
static FREE_HEAD: AtomicI32 = AtomicI32::new(-1);
static COUNT: AtomicI32 = AtomicI32::new(0);
static MAX_COUNT: AtomicI32 = AtomicI32::new(0);
static SET_COUNT: AtomicI32 = AtomicI32::new(1);

#[derive(Error, Debug)]
pub enum TimerError {
    #[error("Failed to lock timer buffer")]
    LockPoisoned,
}

pub fn init(count: i32) -> Result<(), TimerError> {
    let mut data_buffer: Vec<TimerStruct>;

    data_buffer = (0..count).map(|_| TimerStruct::new()).collect();

    data_buffer.iter().for_each(|item| {
        (*TIMER_BUFFER.lock().unwrap()).push(item.clone());
    });

    for index in 0..(count - 1) {
        data_buffer[index as usize].next_timer = index + 1;
    }
    data_buffer[(count - 1) as usize].next_timer = -1;

    FREE_HEAD.store(0, Relaxed);
    ACTIVE_HEAD.store(-1, Relaxed);
    COUNT.store(0, Relaxed);
    MAX_COUNT.store(count, Relaxed);
    SET_COUNT.store(count, Relaxed);

    Ok(())
}

pub fn uninit() {
    let mut buffer = TIMER_BUFFER.lock().unwrap();
    buffer.clear();
    FREE_HEAD.store(-1, Relaxed);
    ACTIVE_HEAD.store(-1, Relaxed);
    COUNT.store(0, Relaxed);
    MAX_COUNT.store(0, Relaxed);
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
    let current_count = COUNT.load(Relaxed);
    let max_count = MAX_COUNT.load(Relaxed);

    if current_count >= max_count {
        return 0;
    }

    let mut buffer = TIMER_BUFFER.lock().unwrap();

    let timer_idx = FREE_HEAD.load(Relaxed);
    if timer_idx == -1 {
        return 0;
    }

    let next_free = buffer[timer_idx as usize].next_timer;
    FREE_HEAD.store(next_free, Relaxed);

    let target_time = (draw_context.time_ticks + (time * 1000.0) as usize) as i32;

    let mut prev = -1;
    let mut current = ACTIVE_HEAD.load(Relaxed);
    let mut index = 0;

    while current != -1
        && index < current_count
        && target_time >= buffer[current as usize].target_time
    {
        prev = current;
        current = buffer[current as usize].next_timer;
        index += 1;
    }

    if prev != -1 {
        buffer[timer_idx as usize].next_timer = buffer[prev as usize].next_timer;
        buffer[prev as usize].next_timer = timer_idx;
    } else {
        buffer[timer_idx as usize].next_timer = ACTIVE_HEAD.load(Relaxed);
        ACTIVE_HEAD.store(timer_idx, Relaxed);
    }

    buffer[timer_idx as usize].caller = caller;
    buffer[timer_idx as usize].callback = Some(callback);
    buffer[timer_idx as usize].target_time = target_time;

    let timer_id = SET_COUNT.load(Relaxed);
    buffer[timer_idx as usize].timer_id = timer_id;

    COUNT.store(current_count + 1, Relaxed);

    let mut set_count = SET_COUNT.load(Relaxed);
    set_count += 1;
    if set_count <= 0 {
        set_count = 1;
    }

    SET_COUNT.store(set_count, Relaxed);

    timer_id
}

pub fn kill_callback(callback: unsafe extern "C" fn(i32, *mut c_void, &mut DrawContext)) -> i32 {
    let mut buffer = TIMER_BUFFER.lock().unwrap();
    let mut count = COUNT.load(Relaxed);
    let mut kill_count = 0;

    let mut current = ACTIVE_HEAD.load(Relaxed);
    let mut prev = -1;

    for index in 0..count {
        if index >= count {
            break;
        }

        let current_idx = current;
        if current_idx == -1 {
            break;
        }

        // TODO: Check this about func ptr comparisons, might not work
        if buffer[current_idx as usize].callback == Some(callback) {
            kill_count += 1;

            if prev != -1 {
                buffer[prev as usize].next_timer = buffer[current_idx as usize].next_timer;
            } else {
                ACTIVE_HEAD.store(buffer[current_idx as usize].next_timer, Relaxed);
            }

            buffer[current_idx as usize].next_timer = FREE_HEAD.load(Relaxed);
            FREE_HEAD.store(current_idx, Relaxed);

            count -= 1;

            if count == index {
                break;
            }

            current = if prev != -1 {
                buffer[prev as usize].next_timer
            } else {
                ACTIVE_HEAD.load(Relaxed)
            };
        } else {
            prev = current;
            current = buffer[current_idx as usize].next_timer;
        }
    }

    COUNT.store(count, Relaxed);
    kill_count
}

pub fn kill_id(timer_id: i32) -> i32 {
    let mut buffer = TIMER_BUFFER.lock().unwrap();
    let mut count = COUNT.load(Relaxed);

    if count <= 0 {
        return 0;
    }

    let mut current = ACTIVE_HEAD.load(Relaxed);
    let mut prev = -1;
    let mut index = 0;

    while current != -1 {
        if buffer[current as usize].timer_id == timer_id {
            break;
        }

        index += 1;
        prev = current;
        current = buffer[current as usize].next_timer;

        if index >= count {
            return 0;
        }
    }

    if prev != -1 {
        buffer[prev as usize].next_timer = buffer[current as usize].next_timer;
    } else {
        ACTIVE_HEAD.store(buffer[current as usize].next_timer, Relaxed);
    }

    buffer[current as usize].next_timer = FREE_HEAD.load(Relaxed);
    FREE_HEAD.store(current, Relaxed);

    count -= 1;
    COUNT.store(count, Relaxed);

    timer_id
}

pub fn check(time_ticks: usize, state: &mut PinballState) -> i32 {
    let mut buffer = TIMER_BUFFER.lock().unwrap();
    let mut count = COUNT.load(Relaxed);
    let mut index = 0;

    let mut current = ACTIVE_HEAD.load(Relaxed);

    if current != -1 {
        while current != -1 && time_ticks as i32 >= buffer[current as usize].target_time {
            count -= 1;

            ACTIVE_HEAD.store(buffer[current as usize].next_timer, Relaxed);

            buffer[current as usize].next_timer = FREE_HEAD.load(Relaxed);
            FREE_HEAD.store(current, Relaxed);

            if let Some(callback) = buffer[current as usize].callback {
                let timer_id = buffer[current as usize].timer_id;
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

            if current == -1 {
                COUNT.store(count, Relaxed);
                return index;
            }
        }

        while current != -1 && time_ticks as i32 >= buffer[current as usize].target_time + 100 {
            count -= 1;

            ACTIVE_HEAD.store(buffer[current as usize].next_timer, Relaxed);

            buffer[current as usize].next_timer = FREE_HEAD.load(Relaxed);
            FREE_HEAD.store(current, Relaxed);

            if let Some(callback) = buffer[current as usize].callback {
                let timer_id = buffer[current as usize].timer_id;
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

    COUNT.store(count, Relaxed);
    index
}
