#![allow(unused)]
#![allow(arithmetic_overflow)]
extern crate core;

use sdl2::sys::{SDL_GetPerformanceCounter, SDL_GetPerformanceFrequency, SDL_GetTicks};
use sdl2::{
    sys::{
        SDL_CreateWindow, SDL_INIT_AUDIO, SDL_INIT_EVENTS, SDL_INIT_GAMECONTROLLER,
        SDL_INIT_JOYSTICK, SDL_INIT_TIMER, SDL_INIT_VIDEO, SDL_Init, SDL_SetMainReady, SDL_Window,
        SDL_WindowFlags::{SDL_WINDOW_HIDDEN, SDL_WINDOW_RESIZABLE},
    },
    video::WindowPos,
};
use std::process::exit;
use std::sync::atomic::AtomicBool;
use std::time::Duration as StdDuration;
use std::time::Instant;
mod fullscrn;
mod gdrv;
mod group_data;
mod loader;
mod maths;
mod options;
mod score;
mod sound;
mod t_ball;
mod t_collision_component;
mod t_edge_segment;
mod t_flipper_edge;
mod t_pinball_component;
mod t_pinball_table;
mod translations;
mod zdrv;

mod midi;
mod pb;
mod render;
mod utils;

pub fn restart_func() {
    todo!();
}

pub struct Duration<const DEN: u64>(i64);
pub struct TimePoint<const DEN: u64>(Duration<DEN>);

pub trait Clock {
    type Duration;
    type TimePoint;
    const IS_STEADY: bool;
    unsafe fn now() -> Self::TimePoint;
}

pub struct SdlPerformanceClock;
impl Clock for SdlPerformanceClock {
    type Duration = Duration<1_000_000_000>;
    type TimePoint = TimePoint<1_000_000_000>;
    const IS_STEADY: bool = true;
    unsafe fn now() -> Self::TimePoint {
        let freq = SDL_GetPerformanceFrequency();
        let ctr = SDL_GetPerformanceCounter();
        let whole = (ctr / freq) * 1_000_000_000;
        let part = (ctr % freq) * 1_000_000_000 / freq;
        TimePoint(Duration((whole + part) as i64))
    }
}

pub struct SdlTickClock;
impl Clock for SdlTickClock {
    type Duration = Duration<1_000>;
    type TimePoint = TimePoint<1_000>;
    const IS_STEADY: bool = true;
    unsafe fn now() -> Self::TimePoint {
        let ticks = SDL_GetTicks();
        TimePoint(Duration(ticks as i64))
    }
}

struct WelfordState {
    pub mean: f64,
    pub m2: f64,
    pub count: i64,
}

impl WelfordState {
    fn new() -> Self {
        Self {
            mean: 0.005,
            m2: 0.0,
            count: 1,
        }
    }

    pub fn advance(&mut self, new_value: f64) {
        self.count += 1;
        let delta = new_value - self.mean;
        self.mean += delta / self.count as f64;
        self.m2 += delta * (new_value - self.mean);
    }

    pub fn get_std_dev(&self) -> f64 {
        return f64::sqrt(self.m2 / (self.count - 1) as f64);
    }
}

fn main() {
    unsafe {
        SDL_SetMainReady();
        if (SDL_Init(
            SDL_INIT_TIMER
                | SDL_INIT_AUDIO
                | SDL_INIT_VIDEO
                | SDL_INIT_EVENTS
                | SDL_INIT_JOYSTICK
                | SDL_INIT_GAMECONTROLLER,
        ) < 0)
        {
            println!("OOPS!! No init, closing");
            exit(1);
        }
    }
    unsafe {
        println!("Creating window");
        let window = SDL_CreateWindow(
            c"PinBall Space Cadet (0.0.0)".as_ptr(),
            0,
            0,
            800,
            556,
            SDL_WINDOW_HIDDEN as u32 | SDL_WINDOW_RESIZABLE as u32,
        );
    }
}
