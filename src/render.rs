use crate::gdrv::GdrvBitmap8;
use crate::maths::RectangleType;
use crate::options::OPTIONS;
use crate::zdrv::ZMapHeaderType;
use crate::{gdrv, zdrv};
use sdl2::sys::SDL_Rect;
use sdl2::sys::SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};
use thiserror::Error;

pub enum VisualTypes {
    Background,
    Sprite,
    Ball,
}

struct RenderSprite {
    bmp_rect: RectangleType, // TODO: List?
    bmp: Option<GdrvBitmap8>,
    zmap: Option<ZMapHeaderType>,
    delete_flag: bool,
    visual_types: VisualTypes,
    depth: u16,
    dirty_rect_prev: RectangleType,
    z_map_offset_y: i32,
    z_map_offset_x: i32,
    dirty_rect: RectangleType,
    occluded_sprites: Option<Vec<Option<RenderSprite>>>, // TODO: Is this really it?
    bounding_rect: RectangleType,
    dirty_flag: bool,
}

pub static V_SCREEN: Mutex<Option<GdrvBitmap8>> = Mutex::new(None);
pub static BACKGROUND_BITMAP: Mutex<Option<GdrvBitmap8>> = Mutex::new(None);
pub static BACKGROUND_ZMAP: Option<ZMapHeaderType> = None;

pub static Z_MAP_OFFSET_X: i32 = 0;
pub static Z_MAP_OFFSET_Y: i32 = 0;

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
}

pub fn init(bmp: Option<GdrvBitmap8>, width: i16, height: i16) -> Result<(), RenderLockError> {
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
    (*v_screen_rect).x_position = 0;
    (*v_screen_rect).y_position = 0;
    (*v_screen_rect).width = width as i32;
    (*v_screen_rect).height = height as i32;

    let mut v_screen_unwrap = (*v_screen).to_owned().unwrap();
    v_screen_unwrap.y_position = 0;
    v_screen_unwrap.x_position = 0;

    let mut ball_bitmap = BALL_BITMAP.lock()?;
    let mut ball_unwrap = ball_bitmap.as_mut().unwrap();
    let mut defaults: [GdrvBitmap8; 20] = std::array::from_fn(|_| GdrvBitmap8::default());
    ball_unwrap = &mut defaults;

    for mut ball_bmp in ball_unwrap.iter_mut() {
        ball_bmp = &mut GdrvBitmap8::new_dims_indexed(64, 64, false);
    }

    *BACKGROUND_BITMAP.lock()? = bmp.clone();
    if bmp.is_some() {
        gdrv::copy_bitmap(
            &mut v_screen_unwrap,
            width as i32,
            height as i32,
            0,
            0,
            bmp.unwrap(),
            0,
            0,
        );
    } else {
        let v_width = v_screen_unwrap.width;
        let v_height = v_screen_unwrap.height;
        gdrv::fill_bitmap(v_screen_unwrap, v_width, v_height, 0, 0, 0);
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

fn paint_balls() {
    todo!()
}

fn unpaint_balls() {
    todo!()
}
