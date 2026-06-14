use crate::gdrv::GdrvBitmap8;
use crate::maths::RectangleType;
use crate::options::OPTIONS;
use crate::zdrv::ZMapHeaderType;
use crate::{gdrv, maths, zdrv};
use sdl2::sys::SDL_Rect;
use sdl2::sys::SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING;
use std::cmp::PartialEq;
use std::rc::Rc;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, PoisonError};
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
            let zmap_bg = BACKGROUND_ZMAP.lock().unwrap();

            instance.zmap = zmap_bg.clone();

            instance.z_map_offset_x = x_pos - *Z_MAP_OFFSET_X.lock().unwrap();
            instance.z_map_offset_y = y_pos - *Z_MAP_OFFSET_Y.lock().unwrap();
        }

        add_sprite(instance.clone());
        instance
    }

    pub fn set(
        &mut self,
        bmp: Option<Arc<GdrvBitmap8>>,
        zmap: Option<Arc<ZMapHeaderType>>,
        x_pos: i32,
        y_pos: i32,
    ) {
    }
}

fn add_sprite(sprite: RenderSprite) {
    let mut list = if sprite.visual_type == VisualTypes::Ball {
        BALL_LIST.lock().unwrap()
    } else {
        SPRITE_LIST.lock().unwrap()
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

pub static V_SCREEN: Mutex<Option<GdrvBitmap8>> = Mutex::new(None);
pub static BACKGROUND_BITMAP: Mutex<Option<GdrvBitmap8>> = Mutex::new(None);
pub static BACKGROUND_ZMAP: Mutex<Option<Arc<ZMapHeaderType>>> = Mutex::new(None);

pub static Z_MAP_OFFSET_X: Mutex<i32> = Mutex::new(0);
pub static Z_MAP_OFFSET_Y: Mutex<i32> = Mutex::new(0);

pub static DESTINATION_RECT: LazyLock<Mutex<SDL_Rect>> = LazyLock::new(|| {
    Mutex::new(SDL_Rect {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    })
});

static SPRITE_LIST: Mutex<Vec<RenderSprite>> = Mutex::new(Vec::new());
static BALL_LIST: Mutex<Vec<RenderSprite>> = Mutex::new(Vec::new());

static OFFSET_X: Mutex<i32> = Mutex::new(0);
static OFFSET_Y: Mutex<i32> = Mutex::new(0);

static V_SCREEN_RECT: LazyLock<Mutex<RectangleType>> =
    LazyLock::new(|| Mutex::new(RectangleType::default()));
static BALL_BITMAP: Mutex<Option<[GdrvBitmap8; 20]>> = Mutex::new(None);

static Z_SCREEN: Mutex<Option<ZMapHeaderType>> = Mutex::new(None);

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

pub fn init(bmp: Option<GdrvBitmap8>, width: i16, height: i16) -> Result<(), RenderLockError> {
    {
        // This block prevents the locks from holding on the next call to recreate_screen_texture();
        let mut v_screen = V_SCREEN.lock()?;
        *v_screen = Some(GdrvBitmap8::new_dims_indexed(
            width as i32,
            height as i32,
            false,
        ));

        let mut z_screen = Z_SCREEN.lock()?;
        *z_screen = Some(ZMapHeaderType::new(
            width as i32,
            height as i32,
            width as i32,
        ));

        let mut z_unwrap = (*z_screen).as_mut().unwrap();
        let z_width = z_unwrap.width;
        let z_height = z_unwrap.height;

        zdrv::fill(z_unwrap, z_width, z_height, 0, 0, 0xFFFF);

        let mut v_screen_rect = V_SCREEN_RECT.lock()?;
        v_screen_rect.x_position = 0;
        v_screen_rect.y_position = 0;
        v_screen_rect.width = width as i32;
        v_screen_rect.height = height as i32;

        let mut v_screen_unwrap = (*v_screen).to_owned().unwrap();
        v_screen_unwrap.y_position = 0;
        v_screen_unwrap.x_position = 0;

        let mut ball_bitmap_guard = BALL_BITMAP.lock()?;
        let mut ball_array = ball_bitmap_guard.get_or_insert_with(|| {
            std::array::from_fn(|_| GdrvBitmap8::new_dims_indexed(64, 64, false))
        });

        *BACKGROUND_BITMAP.lock()? = bmp.clone();
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
    }

    recreate_screen_texture();

    Ok(())
}

pub fn recreate_screen_texture() {
    let mut vscreen = V_SCREEN.lock().unwrap();
    let options = OPTIONS.lock().unwrap();
    let filtering = if *options.linear_filtering {
        c"linear"
    } else {
        c"nearest"
    };
    let v_screen_def = (*vscreen).as_mut().unwrap();

    v_screen_def.create_texture(filtering.as_ptr(), SDL_TEXTUREACCESS_STREAMING as i32);
}

fn repaint(sprite: &RenderSprite) {
    todo!()
}

fn paint_balls() -> Result<(), RenderLockError> {
    let mut ball_list = BALL_LIST.lock()?;
    let v_screen_rect = V_SCREEN_RECT.lock()?;
    let mut v_screen_guard = V_SCREEN.lock()?;
    let z_screen_guard = Z_SCREEN.lock()?;

    let v_screen = (*v_screen_guard).as_mut().unwrap();
    let z_screen = (*z_screen_guard).as_ref().unwrap();

    // Sort ball sprites by ascending depth
    ball_list.sort_by(|a, b| a.depth.cmp(&b.depth));

    // For balls that clip vScreen: save original vScreen contents and paint the ball bitmap.
    for index in 0..ball_list.len() {
        let ball_sprite = &mut ball_list[index];
        let mut ball_guard = BALL_BITMAP.lock()?;
        let mut ball_bitmap = ball_guard.as_mut().unwrap();

        let dirty = &mut ball_sprite.dirty_rect;

        if ball_sprite.bmp.is_some()
            && maths::rectangle_clip(&ball_sprite.bmp_rect, &(*v_screen_rect), dirty)
        {
            let ball_sprite_bmp = ball_sprite.bmp.as_ref().unwrap();

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
                &ball_sprite_bmp,
                x_pos - ball_sprite.bmp_rect.x_position,
                y_pos - ball_sprite.bmp_rect.y_position,
                ball_sprite.depth,
            )
        }
    }

    Ok(())
}

fn unpaint_balls() -> Result<(), RenderLockError> {
    // Restore portions of v_screen saved during previous paint_balls call.
    let mut ball_list = BALL_LIST.lock()?;
    let ball_list_size = (*ball_list).len();

    let mut v_screen_guard = V_SCREEN.lock()?;
    let vscreen = (*v_screen_guard).as_mut().unwrap();

    let mut index = ball_list_size - 1;
    while index >= 0 {
        let cur_ball = &mut ball_list[index];

        let mut ball_bitmap_guard = BALL_BITMAP.lock()?;
        let ball_bitmap = ball_bitmap_guard.as_mut().unwrap();

        if cur_ball.dirty_rect.width > 0 {
            gdrv::copy_bitmap(
                vscreen,
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

        index -= 1;
    }

    Ok(())
}

pub fn update() {
    unpaint_balls();

    let v_screen_rect = V_SCREEN_RECT.lock().unwrap();

    let mut sprite_list = SPRITE_LIST.lock().unwrap();
    // Clip dirty sprites with vScreen, clear clipping (dirty) rectangles
    for sprite in sprite_list.iter_mut() {
        if sprite.dirty_flag {
            continue;
        }

        let mut clear_sprite: bool = false;
        match sprite.visual_type {
            VisualTypes::Background => {
                let rec_clip =
                    maths::rectangle_clip(&sprite.bmp_rect, &v_screen_rect, &mut sprite.dirty_rect);
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
                let rec_clip =
                    maths::rectangle_clip(&dirty_rect, &v_screen_rect, &mut clipped_rect);

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
            let mut z_screen_guard = Z_SCREEN.lock().unwrap();
            let z_screen_mut = z_screen_guard.as_mut().unwrap();
            zdrv::fill(z_screen_mut, width, height, x_pos, y_pos, 0xFFFF);
            let background_bmp = BACKGROUND_BITMAP.lock().unwrap();
            if background_bmp.is_some() {
                let mut bg_bmp = background_bmp.clone().unwrap();
                let mut v_screen_guard = V_SCREEN.lock().unwrap();
                let v_screen = v_screen_guard.as_mut().unwrap();
                gdrv::copy_bitmap(
                    v_screen,
                    width,
                    height,
                    x_pos,
                    y_pos,
                    &mut bg_bmp,
                    x_pos,
                    y_pos,
                );
            } else {
                let mut v_screen_guard = V_SCREEN.lock().unwrap();
                let v_screen = v_screen_guard.as_mut().unwrap();
                gdrv::fill_bitmap(v_screen, width, height, x_pos, y_pos, 0);
            }
        }
    }

    // Paint dirty rectangles of dirty sprites
    for sprite in sprite_list.iter_mut() {
        if sprite.dirty_flag == false {
            continue;
        }

        repaint(sprite);
        sprite.dirty_flag = false;
        sprite.dirty_rect_prev = sprite.dirty_rect;
        if sprite.delete_flag {
            remove_sprite(sprite);
        }
    }

    paint_balls();
}

pub fn remove_sprite(sprite: &RenderSprite) {
    let list = if sprite.visual_type == VisualTypes::Ball {
        &BALL_LIST
    } else {
        &SPRITE_LIST
    };

    // TODO: Arc::ptr_eq
    let mut list_sprites = list.lock().unwrap();
    if let Some(pos) = list_sprites.iter().position(|s| std::ptr::eq(s, sprite)) {
        list_sprites.remove(pos);
    }
}

pub fn set_background_zmap(zmap: Option<Arc<ZMapHeaderType>>, offset_x: i32, offset_y: i32) {
    let mut zmap_guard = BACKGROUND_ZMAP.lock().unwrap();
    *zmap_guard = zmap;
    let mut zmap_offset_x = Z_MAP_OFFSET_X.lock().unwrap();
    let mut zmap_offset_y = Z_MAP_OFFSET_Y.lock().unwrap();
    (*zmap_offset_x) = offset_x;
    (*zmap_offset_y) = offset_y;
}
