use crate::context::component_context::ComponentContext;
use anyhow::Result;
use std::ffi::c_void;
use thiserror::Error;

pub type TimerCallback =
    unsafe extern "C" fn(i32, *mut c_void, &mut ComponentContext) -> Result<()>;

#[derive(Default)]
pub struct TimerManager {
    timers: Vec<Timer>,
    active_head: i32,
    free_head: i32,
    active_count: i32,
    next_id: i32,
    capacity: i32,
}

impl TimerManager {
    pub fn init(&mut self, count: i32) {
        self.timers = vec![Timer::default(); count as usize];

        let len = self.timers.len();

        for (i, timer) in self.timers.iter_mut().enumerate() {
            timer.next = if i + 1 < len { (i + 1) as i32 } else { NONE };
        }

        self.free_head = 0;
        self.active_head = NONE;
        self.active_count = 0;
        self.capacity = count;
        self.next_id = count;
    }

    pub fn set(
        &mut self,
        time: f32,
        caller: *mut c_void,
        callback: TimerCallback,
        component_context: &ComponentContext,
    ) -> Result<i32> {
        if self.active_count >= self.capacity {
            return Ok(0);
        }

        if self.free_head == NONE {
            return Ok(0);
        }

        let timer = self.free_head;
        self.free_head = self.timers[timer as usize].next;
        self.timers[timer as usize].next = NONE;

        let target_time = (component_context.time_ticks + (time * 1000.0) as usize) as i32;

        let mut prev = NONE;
        let mut current = self.active_head;
        let mut index = 0;

        while current != NONE
            && index < self.active_count
            && target_time >= self.timers[current as usize].target_time
        {
            prev = current;
            current = self.timers[current as usize].next;
            index += 1;
        }

        if prev != NONE {
            self.timers[timer as usize].next = self.timers[prev as usize].next;
            self.timers[prev as usize].next = timer;
        } else {
            self.timers[timer as usize].next = self.active_head;
            self.active_head = timer;
        }

        let id = self.next_id;

        self.timers[timer as usize].caller = caller;
        self.timers[timer as usize].callback = Some(callback);
        self.timers[timer as usize].target_time = target_time;
        self.timers[timer as usize].id = id;

        self.active_count += 1;

        self.next_id += 1;
        if self.next_id <= 0 {
            self.next_id = 1;
        }

        Ok(id)
    }

    pub fn kill_callback(&mut self, callback: TimerCallback) -> Result<i32> {
        let mut killed = 0;

        let mut current = self.active_head;
        let mut prev = NONE;

        while current != NONE {
            let current_u = current as usize;

            if self.timers[current_u].callback == Some(callback) {
                killed += 1;
                let next = self.timers[current_u].next;
                self.unlink_active(current, prev);
                current = next;
            } else {
                prev = current;
                current = self.timers[current_u].next;
            }
        }

        Ok(killed)
    }

    pub fn kill_id(&mut self, id: i32) -> Result<i32> {
        if self.active_count <= 0 {
            return Ok(0);
        }

        let mut current = self.active_head;
        let mut prev = NONE;
        let mut index = 0;

        while current != NONE {
            let current_u = current as usize;

            if self.timers[current_u].id == id {
                break;
            }

            index += 1;
            prev = current;
            current = self.timers[current_u].next;

            if index >= self.active_count {
                return Ok(0);
            }
        }

        self.unlink_active(current, prev);

        Ok(id)
    }

    pub fn check(
        &mut self,
        time_ticks: usize,
        component_context: &mut ComponentContext,
    ) -> Result<i32> {
        let mut index = 0;
        let mut current = self.active_head;

        if current != NONE {
            while current != NONE && time_ticks as i32 >= self.timers[current as usize].target_time
            {
                self.active_count -= 1;

                self.active_head = self.timers[current as usize].next;

                self.timers[current as usize].next = self.free_head;
                self.free_head = current;

                self.fire_timer(&self.timers[current as usize], component_context)?;

                current = self.active_head;
                index += 1;

                if index > 1 {
                    break;
                }

                if current == NONE {
                    return Ok(index);
                }
            }
        }

        while current != NONE
            && time_ticks as i32 >= self.timers[current as usize].target_time + 100
        {
            self.active_count -= 1;
            self.active_head = self.timers[current as usize].next;

            self.timers[current as usize].next = self.free_head;
            self.free_head = current;

            self.fire_timer(&self.timers[current as usize], component_context)?;

            current = self.active_head;
            index += 1;
        }

        Ok(index)
    }

    fn fire_timer(&self, timer: &Timer, component_context: &mut ComponentContext) -> Result<()> {
        if let Some(callback) = timer.callback {
            unsafe {
                callback(timer.id, timer.caller, component_context)?;
            }
        }
        Ok(())
    }

    fn unlink_active(&mut self, current: i32, prev: i32) {
        let next = self.timers[current as usize].next;

        if prev != NONE {
            self.timers[prev as usize].next = next;
        } else {
            self.active_head = next;
        }

        self.timers[current as usize].next = self.free_head;
        self.free_head = current;
        self.active_count -= 1;
    }
}

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

#[derive(Error, Debug)]
pub enum TimerError {
    #[error("Failed to lock timer buffer")]
    LockPoisoned,
}
//
// pub fn uninit() {
//     let mut buffer = TIMER_BUFFER.lock().unwrap();
//     buffer.clear();
//     FREE_HEAD.store(NONE, Relaxed);
//     ACTIVE_HEAD.store(NONE, Relaxed);
//     ACTIVE_COUNT.store(0, Relaxed);
//     CAPACITY.store(0, Relaxed);
// }
