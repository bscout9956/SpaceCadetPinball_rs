use crate::MainError::NoneRendererError;
use crate::errors::FullscreenError;
use crate::pinball_state::{OptionsState, PbGameState};
use crate::{MAIN_WINDOW, RENDERER, get_main_menu_height, pb, render};
use sdl2::sys::SDL_WindowFlags::SDL_WINDOW_FULLSCREEN_DESKTOP;
use sdl2::sys::{SDL_GetRendererOutputSize, SDL_Rect, SDL_SetWindowFullscreen};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::{Mutex, atomic};

static RESOLUTION: AtomicI32 = AtomicI32::new(0);

#[derive(Clone)]
pub struct ResolutionInfo {
    screen_width: i16,
    screen_height: i16,
    pub table_width: i16,
    pub table_height: i16,
    resolution_menu_id: i16,
}

pub static SCREEN_MODE: AtomicBool = AtomicBool::new(false);
pub static DISPLAY_CHANGED: AtomicBool = AtomicBool::new(false);
pub static RESOLUTION_ARRAY: Mutex<[ResolutionInfo; 3]> = Mutex::new([
    ResolutionInfo {
        screen_width: 640,
        screen_height: 480,
        table_width: 600,
        table_height: 416,
        resolution_menu_id: 501,
    },
    ResolutionInfo {
        screen_width: 800,
        screen_height: 600,
        table_width: 752,
        table_height: 520,
        resolution_menu_id: 502,
    },
    ResolutionInfo {
        screen_width: 1024,
        screen_height: 768,
        table_width: 960,
        table_height: 666,
        resolution_menu_id: 503,
    },
]);

pub static SCALE_X: Mutex<f32> = Mutex::new(1.0);
pub static SCALE_Y: Mutex<f32> = Mutex::new(1.0);
pub static OFFSET_X: Mutex<f32> = Mutex::new(0.0);
pub static OFFSET_Y: Mutex<f32> = Mutex::new(0.0);

pub fn set_resolution(mut value: i32, pb_game_state: &mut PbGameState) -> Result<(), FullscreenError> {
    if pb_game_state.full_tilt_mode && !pb_game_state.full_tilt_demo_mode {
        value = 0;
    }
    if !(0..=2).contains(&value) {
        return Err(FullscreenError::ResolutionOutOfBounds);
    }
    RESOLUTION.store(value, Relaxed);
    Ok(())
}

pub fn get_max_resolution(pb_game_state: &mut PbGameState) -> i32 {
    if pb_game_state.full_tilt_mode && !pb_game_state.full_tilt_demo_mode {
        2
    } else {
        0
    }
}

pub fn set_screen_mode(is_fullscreen: bool) -> bool {
    let mut result = is_fullscreen;
    let mut screen_mode = SCREEN_MODE.load(Relaxed);

    if is_fullscreen == screen_mode {
        return result;
    }
    screen_mode = is_fullscreen;
    SCREEN_MODE.store(screen_mode, Relaxed);

    if is_fullscreen {
        unsafe {
            match enable_fullscreen() {
                Ok(enabled) => enabled,
                Err(e) => {
                    println!("Failed to enable fullscreen: {}", e);
                    return false;
                }
            }
        };
        return true;
    }

    disable_fullscreen();
    result
}

pub fn get_resolution() -> i32 {
    RESOLUTION.load(atomic::Ordering::Acquire)
}

fn reset_offset(offset: &Mutex<f32>) -> Result<(), FullscreenError> {
    let mut offset = offset
        .lock()
        .map_err(|_| FullscreenError::MainWindowMissing)?;
    *offset = 0.0f32;
    Ok(())
}

pub fn window_size_changed(option_state: &mut OptionsState) -> Result<(), FullscreenError> {
    let (mut width, mut height): (i32, i32) = (0, 0);
    let renderer_guard = RENDERER.lock().map_err(|_| FullscreenError::LockGeneric)?;
    if let Some(renderer) = renderer_guard.as_ref() {
        unsafe {
            SDL_GetRendererOutputSize(renderer.0, &mut width, &mut height);
        }
    } else {
        return Err(FullscreenError::MissingRenderer);
    }

    let menu_height = if *option_state.options.show_menu {
        get_main_menu_height()
    } else {
        0
    };
    height -= menu_height;
    let res = match RESOLUTION_ARRAY.lock() {
        Ok(resolution_array) => {
            let idx = RESOLUTION.load(Relaxed) as usize;
            resolution_array[idx].clone()
        }
        Err(e) => {
            return Err(FullscreenError::ResolutionArrayLock(e));
        }
    };

    update_x_scale(&mut width, &res)?;
    update_y_scale(&mut height, &res)?;

    reset_offset(&OFFSET_X)?;
    reset_offset(&OFFSET_Y)?;

    let mut offset2x = 0;
    let mut offset2y = 0;

    if *option_state.options.integer_scaling {
        let mut scale_x = SCALE_X.lock().map_err(FullscreenError::FloatLock)?;
        let mut scale_y = SCALE_Y.lock().map_err(FullscreenError::FloatLock)?;

        *scale_x = if *scale_x < 1.0 {
            *scale_x
        } else {
            (*scale_x).floor()
        };

        *scale_y = if *scale_y < 1.0 {
            *scale_y
        } else {
            (*scale_y).floor()
        };
    }

    if *option_state.options.uniform_scaling {
        let mut scale_x = SCALE_X.lock().map_err(FullscreenError::FloatLock)?;
        let mut scale_y = SCALE_Y.lock().map_err(FullscreenError::FloatLock)?;
        *scale_x = f32::min(*scale_x, *scale_y);
        *scale_y = *scale_x;
    }

    let scale_x = SCALE_X.lock().map_err(FullscreenError::FloatLock)?;
    let scale_y = SCALE_Y.lock().map_err(FullscreenError::FloatLock)?;
    offset2x = (width as f32 - res.table_width as f32 * *scale_x).floor() as i32;
    offset2y = (height as f32 - res.table_height as f32 * *scale_y).floor() as i32;

    let mut offset_x = OFFSET_X.lock().map_err(FullscreenError::FloatLock)?;
    let mut offset_y = OFFSET_Y.lock().map_err(FullscreenError::FloatLock)?;
    *offset_x = offset2x as f32 / 2.0f32;
    *offset_y = offset2y as f32 / 2.0f32;

    let mut dest_rect = render::DESTINATION_RECT
        .lock()
        .map_err(|_| FullscreenError::LockGeneric)?;

    *dest_rect = SDL_Rect {
        x: *offset_x as i32,
        y: *offset_y as i32 + menu_height,
        w: width - offset2x,
        h: height - offset2y,
    };

    Ok(())
}

fn update_y_scale(height: &mut i32, res: &ResolutionInfo) -> Result<(), FullscreenError> {
    let mut scale_y = SCALE_Y.lock().map_err(FullscreenError::FloatLock)?;
    *scale_y = *height as f32 / res.screen_height as f32;
    Ok(())
}

fn update_x_scale(width: &mut i32, res: &ResolutionInfo) -> Result<(), FullscreenError> {
    let mut scale_x = SCALE_X.lock().map_err(FullscreenError::FloatLock)?;
    *scale_x = *width as f32 / res.screen_width as f32;
    Ok(())
}

pub fn activate(flag: bool) {
    let screen_mode = SCREEN_MODE.load(Relaxed);
    if screen_mode && (!flag) {
        set_screen_mode(false);
    }
}

pub fn shutdown() {
    let display_changed = DISPLAY_CHANGED.load(Relaxed);
    if display_changed {
        set_screen_mode(false);
    }
}

pub fn get_screen_rect_from_pinball_rect(rect: SDL_Rect) -> SDL_Rect {
    // SDL_Rect {
    //     x: 0,
    //     y: 0,
    //     w: 0,
    //     h: 0,
    // }
    todo!()
}

pub fn get_screen_to_pinball_ratio() -> f32 {
    0.0
}

unsafe fn enable_fullscreen() -> Result<bool, FullscreenError> {
    let mut display_changed = DISPLAY_CHANGED.load(Relaxed);
    if !display_changed {
        if let Some(main_window) = MAIN_WINDOW
            .lock()
            .map_err(|_| FullscreenError::LockGeneric)?
            .as_ref()
        {
            unsafe {
                if (SDL_SetWindowFullscreen(main_window.0, SDL_WINDOW_FULLSCREEN_DESKTOP as u32)
                    == 0)
                {
                    display_changed = true;
                    DISPLAY_CHANGED.store(display_changed, Relaxed);
                    return Ok(true);
                }
            }
        } else {
            return Err(FullscreenError::MainWindowMissing);
        }
    }
    Ok(false)
}

fn disable_fullscreen() -> Result<bool, FullscreenError> {
    let mut display_changed = DISPLAY_CHANGED.load(Relaxed);
    if display_changed {
        if let Some(mut main_window) = MAIN_WINDOW
            .lock()
            .map_err(|_| FullscreenError::LockGeneric)?
            .as_ref()
        {
            unsafe {
                if (SDL_SetWindowFullscreen(main_window.0, SDL_WINDOW_FULLSCREEN_DESKTOP as u32)
                    == 0)
                {
                    display_changed = false;
                }
            }
        } else {
            return Err(FullscreenError::MainWindowMissing);
        }
    }

    Ok(false)
}

pub fn init(options_state: &mut OptionsState) {
    window_size_changed(options_state);
}
