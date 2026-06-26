use crate::gdrv::GdrvBitmap8;
use crate::maths::Vector2;
use crate::proj;
use crate::state::debug_state::DebugState;
use crate::state::pb_game_state::PbGameState;
use crate::state::pinball_state::PinballState;
use crate::utils::SdlRendererPtr;
use sdl2::sys::SDL_BlendMode::SDL_BLENDMODE_BLEND;
use sdl2::sys::SDL_TextureAccess::SDL_TEXTUREACCESS_TARGET;
use sdl2::sys::{
    SDL_BlendMode, SDL_GetRenderDrawBlendMode, SDL_GetRenderDrawColor, SDL_GetRenderTarget,
    SDL_RenderClear, SDL_RenderCopy, SDL_RenderDrawLine, SDL_SetRenderDrawBlendMode,
    SDL_SetRenderDrawColor, SDL_SetRenderTarget, SDL_SetTextureBlendMode,
};
use std::ptr::null;

pub(crate) unsafe fn draw_overlay(state: &mut PinballState) {
    if state.debug_state.db_screen.is_none() {
        if let Some(v_screen) = state.render_state.v_screen.as_ref() {
            state.debug_state.db_screen = Some(GdrvBitmap8::new_dims_indexed_buff(
                v_screen.width,
                v_screen.height,
                false,
                false,
            ));
        } else {
            panic!("No screen found, can't debug");
        }

        if let Some(db_screen) = state.debug_state.db_screen.as_mut() {
            db_screen.create_texture(
                "nearest",
                SDL_TEXTUREACCESS_TARGET as i32,
                &state.main_state.renderer,
            );

            let texture = db_screen.texture.as_mut().unwrap().0;

            unsafe {
                SDL_SetTextureBlendMode(texture, SDL_BLENDMODE_BLEND);
            }
        } else {
            panic!("No db_screen found, can't debug");
        }
    }

    let mut initial_r: u8 = 0;
    let mut initial_g: u8 = 0;
    let mut initial_b: u8 = 0;
    let mut initial_a: u8 = 0;

    unsafe {
        if let Some(renderer) = state.main_state.renderer.as_ref() {
            let initial_render_target = SDL_GetRenderTarget(renderer.0);
            SDL_GetRenderDrawColor(
                renderer.0,
                &raw mut initial_r,
                &raw mut initial_g,
                &raw mut initial_b,
                &raw mut initial_a,
            );
            if let Some(db_screen) = state.debug_state.db_screen.as_ref() {
                SDL_SetRenderTarget(renderer.0, db_screen.texture.as_ref().unwrap().0);
                SDL_SetRenderDrawColor(renderer.0, 0, 0, 0, 0);
                SDL_RenderClear(renderer.0);
            }

            // Draw EdgeManager box grid
            if *state.options_state.options.debug_overlay_grid {
                draw_box_grid(renderer, &mut state.pb_game_state);
            }

            // TODO: Rest of debugs
            // if ....

            // Restore render target
            SDL_SetRenderTarget(renderer.0, initial_render_target);
            SDL_SetRenderDrawColor(renderer.0, initial_r, initial_g, initial_b, initial_a);

            let mut blend_mode: SDL_BlendMode = SDL_BlendMode::SDL_BLENDMODE_NONE; // Assigning a random value so we can use it as a ptr

            SDL_GetRenderDrawBlendMode(renderer.0, &raw mut blend_mode);

            SDL_SetRenderDrawBlendMode(renderer.0, SDL_BLENDMODE_BLEND);

            if let Some(db_screen) = state.debug_state.db_screen.as_ref() {
                let dest_rect = state.render_state.destination_rect.unwrap();
                // Is the above potentially null? Might panic
                SDL_RenderCopy(
                    renderer.0,
                    db_screen.texture.as_ref().unwrap().0,
                    null(),
                    &raw const dest_rect,
                );
            }

            SDL_SetRenderDrawBlendMode(renderer.0, blend_mode);
        } else {
            panic!("No renderer found, can't debug");
        }
    }
}

unsafe fn draw_box_grid(renderer: &SdlRendererPtr, state: &mut PbGameState) {
    if let Some(edge_manager) = state.edge_manager.as_ref() {
        unsafe { SDL_SetRenderDrawColor(renderer.0, 0, 255, 0, 255) };
        for x in 0..=edge_manager.max_box_x {
            let mut box_pt = Vector2 {
                x: x as f32 * edge_manager.advance_x + edge_manager.min_x,
                y: edge_manager.min_y,
            };

            let pt_1 = proj::x_form_to_2d(&box_pt);
            box_pt.y = edge_manager.max_box_y as f32 * edge_manager.advance_y + edge_manager.min_y;
            let pt_2 = proj::x_form_to_2d(&box_pt);
            unsafe {
                SDL_RenderDrawLine(renderer.0, pt_1.x, pt_1.y, pt_2.x, pt_2.y);
            }
        }
        for y in 0..=edge_manager.max_box_y {
            let mut box_pt = Vector2 {
                x: edge_manager.min_x,
                y: y as f32 * edge_manager.advance_y + edge_manager.min_y,
            };

            let pt_1 = proj::x_form_to_2d(&box_pt);
            box_pt.x = edge_manager.max_box_x as f32 * edge_manager.advance_x + edge_manager.min_x;
            let pt_2 = proj::x_form_to_2d(&box_pt);

            unsafe {
                SDL_RenderDrawLine(renderer.0, pt_1.x, pt_1.y, pt_2.x, pt_2.y);
            }
        }
    }
}
