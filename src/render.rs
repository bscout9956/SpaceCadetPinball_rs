use crate::gdrv::GdrvBitmap8;
use crate::maths::RectangleType;
use crate::state::main_state::MainState;
use crate::state::options_state::OptionsState;
use crate::state::render_state::RenderState;
use crate::zdrv::ZMapHeaderType;
use crate::{gdrv, maths, zdrv};
use sdl2::sys::SDL_RenderCopy;
use sdl2::sys::SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING;
use std::cmp::PartialEq;
use std::ptr::null;
use std::sync::{Arc, MutexGuard, PoisonError};
use thiserror::Error;

#[derive(PartialEq, Debug, PartialOrd, Ord, Eq, Default, Clone)]
pub enum VisualTypes {
    #[default]
    Background,
    Sprite,
    Ball,
}

#[derive(Default, Clone)]
pub struct RenderSprite {
    pub bmp_rect: RectangleType,
    pub bmp: Option<Arc<GdrvBitmap8>>,
    pub zmap: Option<Arc<ZMapHeaderType>>,
    delete_flag: bool,
    pub visual_type: VisualTypes,
    depth: u16,
    dirty_rect_prev: RectangleType,
    z_map_offset_y: i32,
    z_map_offset_x: i32,
    dirty_rect: RectangleType,
    occluded_sprites: Option<Vec<Option<RenderSprite>>>, // TODO: Is this really it?
    bounding_rect: RectangleType,
    dirty_flag: bool,
}

impl RenderSprite {
    pub fn new(
        visual_type: VisualTypes,
        bmp: Option<Arc<GdrvBitmap8>>,
        zmap: Option<Arc<ZMapHeaderType>>,
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

        if bounding_rect.is_some() {
            instance.bounding_rect = bounding_rect.unwrap();
        } else {
            instance.bounding_rect.width = -1;
            instance.bounding_rect.height = -1;
            instance.bounding_rect.x_position = 0;
            instance.bounding_rect.y_position = 0;
        }

        instance.bmp_rect.x_position = x_pos;
        instance.bmp_rect.y_position = y_pos;

        if let Some(bmp) = instance.bmp.clone() {
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
        bmp: Option<Arc<GdrvBitmap8>>,
        zmap: Option<Arc<ZMapHeaderType>>,
        x_pos: i32,
        y_pos: i32,
    ) {
        let bmp_eq = match (&self.bmp, &bmp) {
            (Some(current), Some(new)) => Arc::ptr_eq(current, new),
            (None, None) => true,
            _ => false,
        };

        let zmap_eq = match (&self.zmap, &zmap) {
            (Some(current), Some(new)) => Arc::ptr_eq(current, new),
            (None, None) => true,
            _ => false,
        };

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

        if let Some(b) = &self.bmp {
            self.bmp_rect.width = b.width;
            self.bmp_rect.height = b.height;
        }
    }
}

fn add_sprite(sprite: RenderSprite, render_state: &mut RenderState) {
    let mut list = if sprite.visual_type == VisualTypes::Ball {
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

#[derive(Debug, Error)]
pub enum RenderLockError {
    #[error("Failed to lock V_SCREEN")]
    VScreen(#[from] PoisonError<MutexGuard<'static, Option<GdrvBitmap8>>>),
    #[error("Failed to lock BALL_BITMAP")]
    BallBitmap(#[from] PoisonError<MutexGuard<'static, Option<[GdrvBitmap8; 20]>>>),
    #[error("Failed to lock Z_SCREEN")]
    ZScreen(#[from] PoisonError<MutexGuard<'static, Option<ZMapHeaderType>>>),
    #[error("Failed to lock RectangleType")]
    Rectangle(#[from] PoisonError<MutexGuard<'static, RectangleType>>),
    #[error("Failed to lock BALL_LIST")]
    BallList(#[from] PoisonError<MutexGuard<'static, Vec<RenderSprite>>>),
}

pub fn init(
    bmp: Option<GdrvBitmap8>,
    width: i16,
    height: i16,
    main_state: &mut MainState,
    options_state: &mut OptionsState,
    render_state: &mut RenderState,
) -> Result<(), RenderLockError> {
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

    let mut z_unwrap = render_state.z_screen.as_mut().unwrap();
    let z_width = z_unwrap.width;
    let z_height = z_unwrap.height;

    zdrv::fill(z_unwrap, z_width, z_height, 0, 0, 0xFFFF);

    render_state.v_screen_rect.x_position = 0;
    render_state.v_screen_rect.y_position = 0;
    render_state.v_screen_rect.width = width as i32;
    render_state.v_screen_rect.height = height as i32;

    let mut v_screen_unwrap = render_state.v_screen.to_owned().unwrap();
    v_screen_unwrap.y_position = 0;
    v_screen_unwrap.x_position = 0;

    let mut ball_array = render_state.ball_bitmap.get_or_insert_with(|| {
        std::array::from_fn(|_| GdrvBitmap8::new_dims_indexed(64, 64, false))
    });

    render_state.background_bitmap = bmp.clone();
    match bmp.is_some() {
        true => {
            gdrv::copy_bitmap(
                &mut v_screen_unwrap,
                width as i32,
                height as i32,
                0,
                0,
                &mut bmp.unwrap(),
                0,
                0,
            );
        }
        false => {
            let v_width = v_screen_unwrap.width;
            let v_height = v_screen_unwrap.height;
            gdrv::fill_bitmap(&mut v_screen_unwrap, v_width, v_height, 0, 0, 0);
        }
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
    let mut clip_rectangle: RectangleType = RectangleType::default();
    if sprite.occluded_sprites.is_none()
        || sprite.visual_type == VisualTypes::Ball
        || sprite.dirty_rect.width <= 0
    {
        return;
    }

    if let Some(sprites) = sprite.occluded_sprites.as_ref() {
        for ref_sprite in sprites {
            if let Some(sprite) = ref_sprite.as_ref()
                && let Some(v_screen) = v_screen.as_mut()
                && let Some(z_screen) = z_screen.as_mut()
                && !sprite.delete_flag
                && sprite.bmp.is_some()
                && maths::rectangle_clip(&sprite.bmp_rect, &sprite.dirty_rect, &mut clip_rectangle)
                && let Some(sprite_bmp) = sprite.bmp.as_ref()
                && let Some(sprite_zmap) = sprite.zmap.as_ref()
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
                    clip_rectangle.x_position - sprite.bmp_rect.x_position,
                    clip_rectangle.y_position - sprite.bmp_rect.y_position,
                    sprite_zmap,
                    clip_rectangle.x_position + sprite.z_map_offset_y - sprite.bmp_rect.x_position,
                    clip_rectangle.y_position + sprite.z_map_offset_x - sprite.bmp_rect.y_position,
                );
            }
        }
    }
}

fn paint_balls(render_state: &mut RenderState) -> Result<(), RenderError> {
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
            && maths::rectangle_clip(&ball_sprite.bmp_rect, &render_state.v_screen_rect, dirty)
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
        }
    }

    Ok(())
}

fn unpaint_balls(render_state: &mut RenderState) -> Result<(), RenderLockError> {
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
                &mut ball_bitmap[index].clone(),
                0,
                0,
            );
        }

        cur_ball.dirty_rect_prev = cur_ball.dirty_rect;
    }

    Ok(())
}

pub fn update(
    render_state: &mut RenderState,
    pb_game_state: &mut PbGameState,
) -> Result<(), RenderError> {
    unpaint_balls(render_state)?;

    // Clip dirty sprites with vScreen, clear clipping (dirty) rectangles
    for sprite in render_state.sprite_list.iter_mut() {
        if sprite.dirty_flag {
            continue;
        }

        let mut clear_sprite: bool = false;
        match sprite.visual_type {
            VisualTypes::Background => {
                let rec_clip = maths::rectangle_clip(
                    &sprite.bmp_rect,
                    &render_state.v_screen_rect,
                    &mut sprite.dirty_rect,
                );
                if rec_clip {
                    clear_sprite = sprite.bmp.is_some();
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
                    &mut clipped_rect,
                );

                if rec_clip {
                    clear_sprite = true;
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
            zdrv::fill(z_screen_mut, width, height, x_pos, y_pos, 0xFFFF);
            if let Some(mut background_bmp) = render_state.background_bitmap.clone() {
                gdrv::copy_bitmap(
                    render_state.v_screen.as_mut().unwrap(),
                    width,
                    height,
                    x_pos,
                    y_pos,
                    &mut background_bmp,
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
                );
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

    // TODO: Arc::ptr_eq
    if let Some(pos) = list.iter().position(|s| std::ptr::eq(s, sprite)) {
        list.remove(pos);
    }
}

pub fn set_background_zmap(
    zmap: Option<Arc<ZMapHeaderType>>,
    offset_x: i32,
    offset_y: i32,
    render_state: &mut RenderState,
) {
    render_state.background_zmap = zmap;
    render_state.z_map_offset_x = offset_x;
    render_state.z_map_offset_y = offset_y;
}

pub(crate) fn present_v_screen(state: &mut PinballState) {
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
                    let table_width_coef = (table.width / v_screen.width) as f32;
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
                            w: 0.0,
                            h: 0.0,
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
                debug_overlay::draw_overlay();
            }
        }
    }
}
