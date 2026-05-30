#![allow(unused)]
#![allow(arithmetic_overflow)]

use std::process::exit;

use sdl2::{
    sys::{
        SDL_CreateWindow, SDL_INIT_AUDIO, SDL_INIT_EVENTS, SDL_INIT_GAMECONTROLLER,
        SDL_INIT_JOYSTICK, SDL_INIT_TIMER, SDL_INIT_VIDEO, SDL_Init, SDL_SetMainReady, SDL_Window,
        SDL_WindowFlags::{SDL_WINDOW_HIDDEN, SDL_WINDOW_RESIZABLE},
    },
    video::WindowPos,
}; // TODO: Remove me when done
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
mod render;
mod utils;

pub fn restart_func() {
    todo!();
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
