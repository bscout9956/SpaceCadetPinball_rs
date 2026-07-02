use crate::gdrv::GdrvBitmap8;
use crate::maths::RectangleType;
use crate::state::main_state::MainState;
use crate::state::options_state::OptionsState;
use crate::state::pb_game_state::PbGameState;
use crate::state::pinball_state::PinballState;
use crate::state::render_state::RenderState;
use crate::zdrv::ZMapHeaderType;
use crate::{debug_overlay, gdrv, maths, utils, zdrv};
use sdl2::sys::SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING;
use sdl2::sys::{SDL_FRect, SDL_RenderCopy, SDL_RenderCopyF};
use std::cmp::PartialEq;
use std::ptr::null;
use std::sync::Arc;

#[derive(PartialEq, Debug, PartialOrd, Ord, Eq, Default, Clone)]
pub enum VisualTypes {
    #[default]
    Background,
    Sprite,
    Ball,
}

#[derive(Default, Clone, Debug)]
pub struct RenderSprite {
    pub bmp_rect: RectangleType,
    pub bmp: Arc<Option<GdrvBitmap8>>,
    pub zmap: Arc<Option<ZMapHeaderType>>,
    delete_flag: bool,
    pub visual_type: VisualTypes,
    depth: u16,
    dirty_rect_prev: RectangleType,
    z_map_offset_y: i32,
    z_map_offset_x: i32,
    dirty_rect: RectangleType,
    occluded_sprites: Option<Vec<Option<RenderSprite>>>,
    bounding_rect: RectangleType,
    dirty_flag: bool,
}

impl RenderSprite {
    pub fn new(
        visual_type: VisualTypes,
        bmp: Arc<Option<GdrvBitmap8>>,
        zmap: Arc<Option<ZMapHeaderType>>,
        x_pos: i32,
        y_pos: i32,
        bounding_rect: Option<RectangleType>,
        render_state: &mut RenderState,
    ) -> Self {
        let dirty_flag = visual_type != VisualTypes::Ball;
        let mut instance = Self {
            bmp_rect: Default::default(),
            bmp,
            zmap,
            delete_flag: false,
            visual_type,
            depth: 0xFFFF,
            dirty_rect_prev: Default::default(),
            z_map_offset_y: 0,
            z_map_offset_x: 0,
            dirty_rect: Default::default(),
            occluded_sprites: None,
            bounding_rect: Default::default(),
            dirty_flag,
        };

        if let Some(rect) = bounding_rect {
            instance.bounding_rect = rect;
        } else {
            instance.bounding_rect.width = -1;
            instance.bounding_rect.height = -1;
            instance.bounding_rect.x_position = 0;
            instance.bounding_rect.y_position = 0;
        }

        instance.bmp_rect.x_position = x_pos;
        instance.bmp_rect.y_position = y_pos;

        if let Some(bmp) = (*instance.bmp).as_ref() {
            instance.bmp_rect.width = bmp.width;
            instance.bmp_rect.height = bmp.height;
        } else {
            instance.bmp_rect.width = 0;
            instance.bmp_rect.height = 0;
        }
        instance.dirty_rect_prev = instance.bmp_rect;

        if instance.zmap.is_none() && instance.visual_type != VisualTypes::Ball {
            unreachable!("Background zmap should not be used");

            instance.zmap = render_state.background_zmap.clone();

            instance.z_map_offset_x = x_pos - render_state.z_map_offset_x;
            instance.z_map_offset_y = y_pos - render_state.z_map_offset_y;
        }

        add_sprite(instance.clone(), render_state);
        instance
    }

    pub fn set(
        &mut self,
        bmp: Arc<Option<GdrvBitmap8>>,
        zmap: Arc<Option<ZMapHeaderType>>,
        x_pos: i32,
        y_pos: i32,
    ) {
        let bmp_eq = Arc::ptr_eq(&self.bmp, &bmp);
        let zmap_eq = Arc::ptr_eq(&self.zmap, &zmap);

        if bmp_eq
            && zmap_eq
            && self.bmp_rect.x_position == x_pos
            && self.bmp_rect.y_position == y_pos
        {
            return;
        }

        self.bmp = bmp;
        self.zmap = zmap;
        self.dirty_flag = self.visual_type != VisualTypes::Ball;
        self.bmp_rect.x_position = x_pos;
        self.bmp_rect.y_position = y_pos;

        if let Some(b) = (*self.bmp).as_ref() {
            self.bmp_rect.width = b.width;
            self.bmp_rect.height = b.height;
        }
    }
}

fn add_sprite(sprite: RenderSprite, render_state: &mut RenderState) {
    let list = if sprite.visual_type == VisualTypes::Ball {
        &mut render_state.ball_list
    } else {
        &mut render_state.sprite_list
    };
    list.push(sprite);
}

impl PartialEq for RenderSprite {
    // NOTE: I am comparing as much as possible since deriving PartialEq isn't trivial
    fn eq(&self, other: &Self) -> bool {
        self.bmp_rect == other.bmp_rect
            && self.depth == other.depth
            && self.visual_type == other.visual_type
            && self.delete_flag == other.delete_flag
            && self.dirty_flag == other.dirty_flag
            && self.z_map_offset_x == other.z_map_offset_x
            && self.z_map_offset_y == other.z_map_offset_y
            && self.dirty_rect == other.dirty_rect
            && self.dirty_rect_prev == other.dirty_rect_prev
            && self.bounding_rect == other.bounding_rect
            && self.bmp.is_some() == other.bmp.is_some()
            && self.zmap.is_some() == other.zmap.is_some()
            && self.occluded_sprites == other.occluded_sprites
    }
}

use anyhow::{Context, Result};

pub fn init(
    bmp: Option<GdrvBitmap8>,
    width: i16,
    height: i16,
    main_state: &mut MainState,
    options_state: &mut OptionsState,
    render_state: &mut RenderState,
    pb_game_state: &mut PbGameState,
) -> Result<()> {
    render_state.v_screen = Some(GdrvBitmap8::new_dims_indexed(
        width as i32,
        height as i32,
        false,
    ));

    render_state.z_screen = Some(ZMapHeaderType::new(
        width as i32,
        height as i32,
        width as i32,
    ));

    let z_unwrap = render_state.z_screen.as_mut().unwrap();
    let z_width = z_unwrap.width;
    let z_height = z_unwrap.height;

    zdrv::fill(z_unwrap, z_width, z_height, 0, 0, 0xFFFF)?;

    render_state.v_screen_rect.x_position = 0;
    render_state.v_screen_rect.y_position = 0;
    render_state.v_screen_rect.width = width as i32;
    render_state.v_screen_rect.height = height as i32;

    let v_screen_unwrap = render_state.v_screen.as_mut().unwrap();
    v_screen_unwrap.y_position = 0;
    v_screen_unwrap.x_position = 0;

    render_state.ball_bitmap.get_or_insert_with(|| {
        std::array::from_fn(|_| GdrvBitmap8::new_dims_indexed(64, 64, false))
    });

    render_state.background_bitmap = bmp.clone();

    if let Some(b) = bmp.as_ref() {
        gdrv::copy_bitmap(v_screen_unwrap, width as i32, height as i32, 0, 0, b, 0, 0);
    } else {
        let v_width = v_screen_unwrap.width;
        let v_height = v_screen_unwrap.height;
        gdrv::fill_bitmap(v_screen_unwrap, v_width, v_height, 0, 0, 0, pb_game_state)
            .context("Failed to fill bitmap for render init")?;
    }

    recreate_screen_texture(main_state, options_state, render_state);

    Ok(())
}

pub fn recreate_screen_texture(
    main_state: &mut MainState,
    options_state: &mut OptionsState,
    render_state: &mut RenderState,
) {
    let filtering = if *options_state.options.linear_filtering {
        "linear"
    } else {
        "nearest"
    };
    let v_screen_def = render_state.v_screen.as_mut().unwrap();

    v_screen_def.create_texture(
        filtering,
        SDL_TEXTUREACCESS_STREAMING as i32,
        &main_state.renderer,
    );
}

fn repaint(
    sprite: &RenderSprite,
    v_screen: &mut Option<GdrvBitmap8>,
    z_screen: &mut Option<ZMapHeaderType>,
) {
    // TODO: Verify this is really correct, do not remove me without actually checking
    let mut clip_rectangle: RectangleType = RectangleType::default();
    if sprite.occluded_sprites.is_none()
        || sprite.visual_type == VisualTypes::Ball
        || sprite.dirty_rect.width <= 0
    {
        return;
    }

    if let Some(sprites) = sprite.occluded_sprites.as_ref() {
        for ref_sprite_opt in sprites {
            if let Some(ref_sprite) = ref_sprite_opt.as_ref()
                && let Some(v_screen) = v_screen.as_mut()
                && let Some(z_screen) = z_screen.as_mut()
                && !sprite.delete_flag
                && sprite.bmp.is_some()
                && maths::rectangle_clip(
                    &ref_sprite.bmp_rect,
                    &sprite.dirty_rect,
                    Some(&mut clip_rectangle),
                )
                && let Some(sprite_bmp) = ref_sprite.bmp.as_ref()
                && let Some(sprite_zmap) = ref_sprite.zmap.as_ref()
            {
                zdrv::paint(
                    clip_rectangle.width,
                    clip_rectangle.height,
                    v_screen,
                    clip_rectangle.x_position,
                    clip_rectangle.y_position,
                    z_screen,
                    clip_rectangle.x_position,
                    clip_rectangle.y_position,
                    sprite_bmp,
                    clip_rectangle.x_position - ref_sprite.bmp_rect.x_position,
                    clip_rectangle.y_position - ref_sprite.bmp_rect.y_position,
                    sprite_zmap,
                    // TODO: Original code does this, shouldn't it be using X X X instead of X Y X??
                    clip_rectangle.x_position + ref_sprite.z_map_offset_y
                        - ref_sprite.bmp_rect.x_position,
                    clip_rectangle.y_position + ref_sprite.z_map_offset_x
                        - ref_sprite.bmp_rect.y_position,
                );
            }
        }
    }
}

fn paint_balls(render_state: &mut RenderState) -> Result<()> {
    let v_screen = render_state.v_screen.as_mut().unwrap();
    let z_screen = render_state.z_screen.as_ref().unwrap();

    // Sort ball sprites by ascending depth
    render_state.ball_list.sort_by_key(|a| a.depth);

    // For balls that clip vScreen: save original vScreen contents and paint the ball bitmap.
    for index in 0..render_state.ball_list.len() {
        let ball_sprite = &mut render_state.ball_list[index];
        let ball_bitmap = render_state.ball_bitmap.as_mut().unwrap();

        let dirty = &mut ball_sprite.dirty_rect;

        if let Some(src_bmp) = ball_sprite.bmp.as_ref()
            && maths::rectangle_clip(
                &ball_sprite.bmp_rect,
                &render_state.v_screen_rect,
                Some(dirty),
            )
        {
            let x_pos = dirty.x_position;
            let y_pos = dirty.y_position;
            gdrv::copy_bitmap(
                &mut ball_bitmap[index],
                dirty.width,
                dirty.height,
                0,
                0,
                v_screen,
                x_pos,
                y_pos,
            );

            zdrv::paint_flat(
                dirty.width,
                dirty.height,
                v_screen,
                x_pos,
                y_pos,
                z_screen,
                x_pos,
                y_pos,
                src_bmp,
                x_pos - ball_sprite.bmp_rect.x_position,
                y_pos - ball_sprite.bmp_rect.y_position,
                ball_sprite.depth,
            )
        } else {
            dirty.width = -1;
        }
    }

    Ok(())
}

fn unpaint_balls(render_state: &mut RenderState) -> Result<()> {
    // Restore portions of v_screen saved during previous paint_balls call.
    let ball_list_size = render_state.ball_list.len();

    for index in (0..ball_list_size).rev() {
        let cur_ball = &mut render_state.ball_list[index];

        let ball_bitmap = render_state.ball_bitmap.as_mut().unwrap();

        if cur_ball.dirty_rect.width > 0 {
            gdrv::copy_bitmap(
                render_state.v_screen.as_mut().unwrap(),
                cur_ball.dirty_rect.width,
                cur_ball.dirty_rect.height,
                cur_ball.dirty_rect.x_position,
                cur_ball.dirty_rect.y_position,
                &ball_bitmap[index].clone(),
                0,
                0,
            );
        }

        cur_ball.dirty_rect_prev = cur_ball.dirty_rect;
    }

    Ok(())
}

pub fn update(render_state: &mut RenderState, pb_game_state: &mut PbGameState) -> Result<()> {
    unpaint_balls(render_state)?;

    // Clip dirty sprites with vScreen, clear clipping (dirty) rectangles
    for sprite in render_state.sprite_list.iter_mut() {
        if !sprite.dirty_flag {
            continue;
        }

        let mut clear_sprite: bool = false;
        match sprite.visual_type {
            VisualTypes::Background => {
                let rec_clip = maths::rectangle_clip(
                    &sprite.bmp_rect,
                    &render_state.v_screen_rect,
                    Some(&mut sprite.dirty_rect),
                );
                if rec_clip {
                    clear_sprite = sprite.bmp.is_none();
                } else {
                    sprite.dirty_rect.width = -1;
                }
            }
            VisualTypes::Sprite => {
                if sprite.dirty_rect_prev.width > 0 {
                    maths::enclosing_box(
                        &sprite.dirty_rect_prev,
                        &sprite.bmp_rect,
                        &mut sprite.dirty_rect,
                    );
                }

                let dirty_rect = sprite.dirty_rect;
                let mut clipped_rect = dirty_rect;

                let rec_clip = maths::rectangle_clip(
                    &dirty_rect,
                    &render_state.v_screen_rect,
                    Some(&mut clipped_rect),
                );

                if rec_clip {
                    clear_sprite = true;
                    sprite.dirty_rect = clipped_rect;
                } else {
                    sprite.dirty_rect.width = -1;
                }
            }
            _ => {}
        }

        if clear_sprite {
            let y_pos = sprite.dirty_rect.y_position;
            let width = sprite.dirty_rect.width;
            let x_pos = sprite.dirty_rect.x_position;
            let height = sprite.dirty_rect.height;

            let z_screen_mut = render_state.z_screen.as_mut().unwrap();
            zdrv::fill(z_screen_mut, width, height, x_pos, y_pos, 0xFFFF)
                .context("Failed to fill zdrv for render update")?;
            if let Some(background_bmp) = render_state.background_bitmap.clone() {
                gdrv::copy_bitmap(
                    render_state.v_screen.as_mut().unwrap(),
                    width,
                    height,
                    x_pos,
                    y_pos,
                    &background_bmp,
                    x_pos,
                    y_pos,
                );
            } else {
                gdrv::fill_bitmap(
                    render_state.v_screen.as_mut().unwrap(),
                    width,
                    height,
                    x_pos,
                    y_pos,
                    0,
                    pb_game_state,
                )
                .context("Failed to fill sprite bitmap for render update:")?;
            }
        }
    }

    let mut sprites_to_remove = Vec::new();

    for (index, sprite) in render_state.sprite_list.iter_mut().enumerate() {
        if !sprite.dirty_flag {
            continue;
        }
        repaint(
            sprite,
            &mut render_state.v_screen,
            &mut render_state.z_screen,
        );
        sprite.dirty_flag = false;
        sprite.dirty_rect_prev = sprite.dirty_rect;

        if sprite.delete_flag {
            sprites_to_remove.push(index);
        }
    }

    for index in sprites_to_remove.into_iter().rev() {
        let isolated_sprite = render_state.sprite_list.remove(index);
        remove_sprite(&isolated_sprite, render_state);
    }

    paint_balls(render_state)?;
    Ok(())
}

pub fn remove_sprite(sprite: &RenderSprite, render_state: &mut RenderState) {
    let list = if sprite.visual_type == VisualTypes::Ball {
        &mut render_state.ball_list
    } else {
        &mut render_state.sprite_list
    };

    if let Some(pos) = list.iter().position(|s| s == sprite) {
        list.remove(pos);
    }
}

pub fn set_background_zmap(
    zmap: Arc<Option<ZMapHeaderType>>,
    offset_x: i32,
    offset_y: i32,
    render_state: &mut RenderState,
) {
    render_state.background_zmap = zmap;
    render_state.z_map_offset_x = offset_x;
    render_state.z_map_offset_y = offset_y;
}

pub(crate) fn present_v_screen(state: &mut PinballState) -> Result<()> {
    if let Some(v_screen) = state.render_state.v_screen.as_mut() {
        unsafe {
            v_screen.blit_to_texture();

            if state.render_state.offset_x == 0
                && state.render_state.offset_y == 0
                && let Some(renderer) = state.main_state.renderer.as_ref()
                && let Some(tex) = v_screen.texture.as_mut()
                && let Some(dest_rect) = state.render_state.destination_rect.as_ref()
            {
                SDL_RenderCopy(renderer.0, tex.0, null(), dest_rect);
            } else {
                if let Some(table) = state.pb_game_state.main_table.as_ref() {
                    let table_width_coef = table.borrow().width as f32 / v_screen.width as f32;
                    let src_separation_x =
                        f32::round(v_screen.width as f32 * table_width_coef) as i32;
                    let src_board_rect = SDL_Rect {
                        x: 0,
                        y: 0,
                        w: src_separation_x,
                        h: v_screen.height,
                    };
                    let src_sidebar_rect = SDL_Rect {
                        x: src_separation_x,
                        y: 0,
                        w: v_screen.width - src_separation_x,
                        h: v_screen.height,
                    };

                    if let Some(destination_rect) = state.render_state.destination_rect.as_ref() {
                        let dst_separation_x = destination_rect.w as f32 * table_width_coef;
                        let dst_board_rect = SDL_FRect {
                            x: destination_rect.x as f32
                                + state.render_state.offset_x as f32 * state.fullscrn_state.scale_x,
                            y: destination_rect.y as f32
                                + state.render_state.offset_y as f32 * state.fullscrn_state.scale_y,
                            w: dst_separation_x,
                            h: destination_rect.h as f32,
                        };
                        let dst_sidebar_rect = SDL_FRect {
                            x: destination_rect.x as f32 + dst_separation_x,
                            y: destination_rect.y as f32,
                            w: destination_rect.w as f32 - dst_separation_x,
                            h: destination_rect.h as f32,
                        };

                        if let Some(renderer) = &state.main_state.renderer
                            && let Some(texture) = v_screen.texture.as_mut()
                        {
                            SDL_RenderCopyF(
                                renderer.0,
                                texture.0,
                                &raw const src_board_rect,
                                &raw const dst_board_rect,
                            );
                            SDL_RenderCopyF(
                                renderer.0,
                                texture.0,
                                &raw const src_sidebar_rect,
                                &raw const dst_sidebar_rect,
                            );
                        }
                    }
                }
            }

            if *state.options_state.options.debug_overlay {
                debug_overlay::draw_overlay(state);
            }
        }
    } else {
        eprintln!("No vscreen to present");
    }
}

pub(crate) fn build_occlude_list(state: &mut RenderState) {
    for sprite in &mut state.sprite_list {
        sprite.occluded_sprites = None;
    }

    let num_sprites = state.sprite_list.len();

    let mut all_occlusions: Vec<Option<Vec<usize>>> = vec![None; num_sprites];

    let mut current_occlusions: Vec<usize> = Vec::new();

    for i in 0..num_sprites {
        let main_sprite = &state.sprite_list[i];

        if main_sprite.delete_flag || main_sprite.bounding_rect.width == -1 {
            continue;
        }

        current_occlusions.clear();

        for j in 0..num_sprites {
            let ref_sprite = &state.sprite_list[j];

            if !ref_sprite.delete_flag
                && ref_sprite.bounding_rect.width != -1
                && maths::rectangle_clip(
                    &main_sprite.bounding_rect,
                    &ref_sprite.bounding_rect,
                    None,
                )
            {
                current_occlusions.push(j);
            }
        }

        let has_bmp = main_sprite.bmp.is_some();
        if has_bmp && current_occlusions.len() < 2 {
            current_occlusions.clear();
        }

        if !current_occlusions.is_empty() {
            all_occlusions[i] = Some(current_occlusions.clone());
        }
    }

    for i in 0..state.sprite_list.len() {
        if let Some(indices) = all_occlusions[i].take() {
            let cloned_sprites: Vec<Option<RenderSprite>> = indices
                .into_iter()
                .map(|idx| Some(state.sprite_list[idx].clone()))
                .collect();

            state.sprite_list[i].occluded_sprites = Some(cloned_sprites);
        }
    }
}
