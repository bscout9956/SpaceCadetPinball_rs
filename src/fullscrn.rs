use crate::MainError::NoneRendererError;
use crate::errors::FullscreenError;
use crate::pinball_state::{FullscrnState, MainState, OptionsState, PbGameState};
use crate::{MAIN_WINDOW, pb, render};
use sdl2::sys::SDL_WindowFlags::SDL_WINDOW_FULLSCREEN_DESKTOP;
use sdl2::sys::{SDL_GetRendererOutputSize, SDL_Rect, SDL_SetWindowFullscreen};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::{Mutex, atomic};

static RESOLUTION: AtomicI32 = AtomicI32::new(0);

#[derive(Clone)]
pub struct ResolutionInfo {
    pub(crate) screen_width: i16,
    pub(crate) screen_height: i16,
    pub table_width: i16,
    pub table_height: i16,
    pub(crate) resolution_menu_id: i16,
}

pub fn set_resolution(
    mut value: i32,
    pb_game_state: &mut PbGameState,
) -> Result<(), FullscreenError> {
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

pub fn set_screen_mode(is_fullscreen: bool, fullscrn_state: &mut FullscrnState) -> bool {
    let mut result = is_fullscreen;

    if is_fullscreen == fullscrn_state.screen_mode {
        return result;
    }
    fullscrn_state.screen_mode = is_fullscreen;

    if is_fullscreen {
        unsafe {
            match enable_fullscreen(fullscrn_state) {
                Ok(enabled) => enabled,
                Err(e) => {
                    println!("Failed to enable fullscreen: {}", e);
                    return false;
                }
            }
        };
        return true;
    }

    disable_fullscreen(fullscrn_state);
    result
}

pub fn get_resolution() -> i32 {
    RESOLUTION.load(atomic::Ordering::Acquire)
}

fn reset_offset(mut offset: f32) {
    offset = 0.0f32;
}

pub fn window_size_changed(
    fullscrn_state: &mut FullscrnState,
    main_state: &mut MainState,
    option_state: &mut OptionsState,
) -> Result<(), FullscreenError> {
    let (mut width, mut height): (i32, i32) = (0, 0);
    if let Some(renderer) = main_state.renderer.as_ref() {
        unsafe {
            SDL_GetRendererOutputSize(renderer.0, &mut width, &mut height);
        }
    } else {
        return Err(FullscreenError::MissingRenderer);
    }

    let menu_height = if *option_state.options.show_menu {
        main_state.main_menu_height
    } else {
        0
    };
    height -= menu_height;

    let res = &fullscrn_state.resolution_array[RESOLUTION.load(Relaxed) as usize];

    update_x_scale(&mut width, res, fullscrn_state.scale_x);
    update_y_scale(&mut height, res, fullscrn_state.scale_y);

    reset_offset(fullscrn_state.offset_x);
    reset_offset(fullscrn_state.offset_y);

    let mut offset_2x = 0;
    let mut offset_2y = 0;

    if *option_state.options.integer_scaling {
        fullscrn_state.scale_x = if fullscrn_state.scale_x < 1.0 {
            fullscrn_state.scale_x
        } else {
            fullscrn_state.scale_x.floor()
        };

        fullscrn_state.scale_y = if fullscrn_state.scale_y < 1.0 {
            fullscrn_state.scale_y
        } else {
            fullscrn_state.scale_y.floor()
        };
    }

    if *option_state.options.uniform_scaling {
        fullscrn_state.scale_x = f32::min(fullscrn_state.scale_x, fullscrn_state.scale_y);
        fullscrn_state.scale_y = fullscrn_state.scale_x;
    }

    offset_2x = (width as f32 - res.table_width as f32 * fullscrn_state.scale_x).floor() as i32;
    offset_2y = (height as f32 - res.table_height as f32 * fullscrn_state.scale_y).floor() as i32;

    fullscrn_state.offset_x = offset_2x as f32 / 2.0f32;
    fullscrn_state.offset_y = offset_2y as f32 / 2.0f32;

    let mut dest_rect = render::DESTINATION_RECT
        .lock()
        .map_err(|_| FullscreenError::LockGeneric)?;

    *dest_rect = SDL_Rect {
        x: fullscrn_state.offset_x as i32,
        y: fullscrn_state.offset_y as i32 + menu_height,
        w: width - offset_2x,
        h: height - offset_2y,
    };

    Ok(())
}

fn update_y_scale(height: &mut i32, res: &ResolutionInfo, mut scale_y: f32) {
    scale_y = *height as f32 / res.screen_height as f32;
}

fn update_x_scale(width: &mut i32, res: &ResolutionInfo, mut scale_x: f32) {
    scale_x = *width as f32 / res.screen_width as f32;
}

pub fn activate(flag: bool, fullscrn_state: &mut FullscrnState) {
    if fullscrn_state.screen_mode && (!flag) {
        set_screen_mode(false, fullscrn_state);
    }
}

pub fn shutdown(fullscrn_state: &mut FullscrnState) {
    if fullscrn_state.display_changed {
        set_screen_mode(false, fullscrn_state);
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

unsafe fn enable_fullscreen(fullscrn_state: &mut FullscrnState) -> Result<bool, FullscreenError> {
    if !fullscrn_state.display_changed {
        if let Some(main_window) = MAIN_WINDOW
            .lock()
            .map_err(|_| FullscreenError::LockGeneric)?
            .as_ref()
        {
            unsafe {
                if (SDL_SetWindowFullscreen(main_window.0, SDL_WINDOW_FULLSCREEN_DESKTOP as u32)
                    == 0)
                {
                    fullscrn_state.display_changed = true;
                    return Ok(true);
                }
            }
        } else {
            return Err(FullscreenError::MainWindowMissing);
        }
    }
    Ok(false)
}

fn disable_fullscreen(fullscrn_state: &mut FullscrnState) -> Result<bool, FullscreenError> {
    if fullscrn_state.display_changed {
        if let Some(mut main_window) = MAIN_WINDOW
            .lock()
            .map_err(|_| FullscreenError::LockGeneric)?
            .as_ref()
        {
            unsafe {
                if (SDL_SetWindowFullscreen(main_window.0, SDL_WINDOW_FULLSCREEN_DESKTOP as u32)
                    == 0)
                {
                    fullscrn_state.display_changed = false;
                }
            }
        } else {
            return Err(FullscreenError::MainWindowMissing);
        }
    }

    Ok(false)
}

pub fn init(
    fullscrn_state: &mut FullscrnState,
    main_state: &mut MainState,
    options_state: &mut OptionsState,
) {
    window_size_changed(fullscrn_state, main_state, options_state);
}
