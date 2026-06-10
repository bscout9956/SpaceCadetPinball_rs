use crate::errors::PbError;
use crate::fullscrn::RESOLUTION_ARRAY;
use crate::gdrv::{ColorRgba, GdrvBitmap8};
use crate::group_data::{DatFile, EntryBuffer, FieldTypes};
use crate::maths::{Vector2, Vector3, normalize_2d};
use crate::message_code::MessageCode;
use crate::options::OPTIONS;
use crate::t_collision_component::ICollisionComponent;
use crate::t_pinball_table::TPinballTable;
use crate::t_textbox::TTextBox;
use crate::translations::{Msg, TranslationError};
use crate::{
    DEMO_ACTIVE, HIGH_SCORES_ENABLED, LAUNCH_BALL_ENABLED, MAIN_WINDOW, control, fullscrn, gdrv,
    loader, maths, midi, partman, proj, render, score, t_table_layer, timer, translations,
};
use sdl2::sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_ERROR;
use sdl2::sys::{SDL_MessageBoxFlags, SDL_ShowMessageBox, SDL_ShowSimpleMessageBox};
use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char};
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{LazyLock, Mutex};

pub static QUICK_FLAG: AtomicBool = AtomicBool::new(false);
pub static FULL_TILT_MODE: AtomicBool = AtomicBool::new(false);

pub static FULL_TILT_DEMO_MODE: AtomicBool = AtomicBool::new(false);

pub static CHEAT_MODE: AtomicBool = AtomicBool::new(false);

pub static DEMO_MODE: AtomicBool = AtomicBool::new(false);

pub static CREDITS_ACTIVE: AtomicBool = AtomicBool::new(false);

pub static IDLE_TIMER_MS: Mutex<f32> = Mutex::new(0.0);

pub static TIME_TICKS: AtomicUsize = AtomicUsize::new(0);

pub static DAT_FILE_NAME: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static BASE_PATH: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));
pub static RECORD_TABLE: LazyLock<Mutex<Option<DatFile>>> = LazyLock::new(|| Mutex::new(None));

pub static MAIN_TABLE: LazyLock<Mutex<Option<TPinballTable>>> = LazyLock::new(|| Mutex::new(None));

pub static MISS_TEXT_BOX: Mutex<Option<TTextBox>> = Mutex::new(None);

pub static GAME_MODE: Mutex<GameModes> = Mutex::new(GameModes::GameOver);

static TIME_NEXT: Mutex<f32> = Mutex::new(0.0);

static TIME_NOW: Mutex<f32> = Mutex::new(0.0);

#[derive(PartialEq, Eq, Ord, PartialOrd)]
pub enum GameModes {
    InGame = 1,
    GameOver = 2,
}

pub fn make_path_name(file_name: &str) -> String {
    match BASE_PATH.lock() {
        Ok(path) => {
            return format!("{}{}", *path, file_name);
        }
        Err(e) => {
            println!("Failed to lock base_path {}", e);
        }
    }
    String::new()
}

pub fn get_rc_string(u_id: Msg) -> Result<&'static str, TranslationError> {
    translations::get_translation(u_id)
}

pub fn show_message_box(
    flags: SDL_MessageBoxFlags,
    title: &str,
    message: &str,
) -> Result<(), PbError> {
    if flags == SDL_MESSAGEBOX_ERROR {
        write!(std::io::stderr(), "BL error {}\n{}\n", title, message).unwrap();
    } else {
        write!(std::io::stdout(), "BL error {}\n{}\n", title, message).unwrap();
    }

    let title_cstr = CStr::from_bytes_with_nul(title.as_bytes()).unwrap();
    let message_cstr = CStr::from_bytes_with_nul(message.as_bytes()).unwrap();

    let mut main_window = MAIN_WINDOW.lock().map_err(|_| PbError::LockGeneric)?;
    let main_window_ptr = main_window.as_mut().unwrap();

    unsafe {
        SDL_ShowSimpleMessageBox(
            flags as u32,
            title_cstr.as_ptr(),
            message_cstr.as_ptr(),
            main_window_ptr,
        );
    }

    Ok(())
}

pub fn show_message_box_cstr_message(
    flags: SDL_MessageBoxFlags,
    title: &str,
    message: *const c_char,
) {
    let message_str = unsafe { CStr::from_ptr(message).to_str().unwrap() };
    show_message_box(flags, title, message_str);
}

pub fn select_dat_file(data_search_paths: &[&str]) {
    clear_dat_file_name();
    FULL_TILT_MODE.store(false, Relaxed);
    FULL_TILT_DEMO_MODE.store(false, Relaxed);

    let mut dat_file_names: [&str; 3] = ["CADET.DAT", "PINBALL.DAT", "DEMO.DAT"];

    match OPTIONS.lock() {
        Ok(mut options) => {
            if options.prefer_3dpb_game_data.value {
                dat_file_names.swap(0, 1);
            }
        }
        Err(e) => {
            println!("Error locking OPTIONS: {}", e);
        }
    }

    for path in data_search_paths {
        if path.is_empty() {
            continue;
        }

        set_base_path(path);

        for dat_file_name in dat_file_names {
            let mut file_name = dat_file_name.to_string();
            for i in 0..2 {
                if i == 1 {
                    file_name = file_name.to_lowercase();
                }

                let dat_file_path = make_path_name(&file_name);
                if let Err(e) = File::open(&dat_file_path) {
                    println!("Error opening dat_file {}: {}", &dat_file_path, e);
                    continue;
                }
                set_dat_file_name(&file_name);

                update_full_tilt_mode(dat_file_name);
                return;
            }
        }
    }
}

fn clear_dat_file_name() {
    match DAT_FILE_NAME.lock() {
        Ok(mut file_name) => {
            file_name.clear();
        }
        Err(e) => {
            println!("Error locking DAT_FILE_NAME: {}", e);
        }
    }
}

fn set_dat_file_name(file_name: &str) {
    match DAT_FILE_NAME.lock() {
        Ok(mut dat_name) => {
            *dat_name = String::from(file_name);
        }
        Err(e) => {
            println!("Error locking DAT_FILE_NAME: {}", e);
        }
    }
}

fn update_full_tilt_mode(dat_file_name: &str) {
    if dat_file_name == "CADET.DAT" {
        FULL_TILT_MODE.store(true, Relaxed);
    }
    if dat_file_name == "DEMO.DAT" {
        FULL_TILT_MODE.store(true, Relaxed);
        FULL_TILT_DEMO_MODE.store(true, Relaxed);
    }
}

fn set_base_path(path: &str) {
    match BASE_PATH.lock() {
        Ok(mut base_path) => {
            *base_path = String::from(path);
        }
        Err(e) => {
            println!("Error locking BASE_PATH: {}", e);
        }
    }
}

fn read_camera_floats(float_data: &[u8]) -> Vec<f32> {
    let mut data: Vec<f32> = Vec::new();

    for i in 0..12 {
        let offset = i * 4;
        data.push(f32::from_le_bytes([
            float_data[offset],
            float_data[offset + 1],
            float_data[offset + 2],
            float_data[offset + 3],
        ]));
    }

    data
}

pub fn init() -> Result<(bool), PbError> {
    let mut projection_matrix: [f32; 12] = [0.0; 12];

    let mut data_file_path = String::new();

    match DAT_FILE_NAME.lock() {
        Ok(mut file_name) => {
            if file_name.is_empty() {
                return Ok(false);
            }
            data_file_path = make_path_name(&file_name);
        }
        Err(e) => {
            println!("Error locking DAT_FILE_NAME: {}", e);
            return Ok(false);
        }
    }

    match RECORD_TABLE.lock() {
        Ok(mut record_table) => {
            *record_table = Some(partman::load_records(
                data_file_path,
                FULL_TILT_MODE.load(Relaxed),
            )?);
        }
        Err(e) => {
            println!("Error locking RECORD_TABLE: {}", e);
        }
    }

    let use_bmp_font: i32 = get_rc_int(Msg::TextBoxUseBitmapFont)?;
    if use_bmp_font == 1 {
        score::load_msg_font("pbmsg_ft");
    }

    match RECORD_TABLE.lock() {
        Ok(mut record_table) => {
            if record_table.is_none() {
                return Ok(true);
            } else {
                let table = record_table.as_mut().unwrap();
                let plt = table.field_labeled("background", FieldTypes::Palette);
                let plt_data = plt.unwrap();
                match plt_data {
                    EntryBuffer::Raw(data) => {
                        let mut palette_colors = Vec::with_capacity(256);
                        // extract method here
                        for i in 0..256 {
                            let offset = i * 4;
                            if offset + 3 < data.len() {
                                let color = u32::from_le_bytes([
                                    data[offset],
                                    data[offset + 1],
                                    data[offset + 2],
                                    data[offset + 3],
                                ]);
                                palette_colors.push(ColorRgba::color_rgba_u32(color));
                            } else {
                                palette_colors.push(ColorRgba::black());
                            }
                        }

                        gdrv::display_palette(Some(&palette_colors));
                    }
                    _ => {}
                }

                let mut background_bmp = table
                    .get_bitmap(table.record_labeled("background"))
                    .to_owned();
                let camera_info_id =
                    table.record_labeled("camera_info") + fullscrn::get_resolution();
                let camera_data = table.field(camera_info_id, FieldTypes::FloatArray).unwrap();
                let mut camera_info: Vec<f32> = Vec::new();
                match camera_data {
                    EntryBuffer::Raw(float_data) => {
                        camera_info = read_camera_floats(float_data);
                    }
                    _ => {}
                }
                let res_array = RESOLUTION_ARRAY.lock().unwrap();
                let res_info = &(*res_array)[fullscrn::get_resolution() as usize];

                if !camera_info.is_empty() {
                    projection_matrix.copy_from_slice(&camera_info);

                    let proj_center_x = res_info.table_width as f32 * 0.5;
                    let proj_center_y = res_info.table_height as f32 * 0.5;
                    let proj_d = camera_info[0];
                    let z_min = camera_info[1];
                    let z_scaler = camera_info[2];
                    proj::init(
                        projection_matrix,
                        proj_d,
                        proj_center_x,
                        proj_center_y,
                        z_min,
                        z_scaler,
                    );
                }

                render::init(None, res_info.table_width, res_info.table_height);

                let mut v_guard = render::V_SCREEN.lock().unwrap();
                if let Some(ref mut dst) = *v_guard {
                    gdrv::copy_bitmap(
                        dst,
                        background_bmp.width,
                        background_bmp.height,
                        background_bmp.x_position,
                        background_bmp.y_position,
                        &mut background_bmp,
                        0,
                        0,
                    );
                }

                loader::load_from(table)?;
            }
        }
        Err(e) => {
            println!("Error locking RECORD_TABLE {}", e);
        }
    }

    // TODO: Implement modechange and gamemodes
    //mode_change(GameModes::InGame);

    TIME_TICKS.store(0, Relaxed);
    // TODO: Implement timer init
    //timer::init(150);
    // TODO: Implement score init
    //score::init();

    Ok(true)
}

// Note: This used to be code that took a string like "1 Blablabla" and would get only the first part of the string
// It's an old CPP programming practice apparently,
// I guess valued enums weren't a thing? I am not quite sure
pub fn get_rc_int(u_id: Msg) -> Result<i32, TranslationError> {
    let s = get_rc_string(u_id)?;

    let first_char = s.split_whitespace().next().unwrap_or("0");

    Ok(first_char.parse::<i32>().unwrap_or(0))
}

pub fn reset_table() -> Result<(), PbError> {
    let mut table_opt = MAIN_TABLE.lock()?;
    match table_opt.as_mut() {
        Some(mut main_table) => {
            main_table.message(MessageCode::RESET, 0.0);
            Ok(())
        }
        None => Ok(()),
    }
}

pub fn first_time_setup() {
    render::update();
}

pub(crate) fn toggle_demo() {
    todo!()
}

pub fn replay_level(demo_mode: bool) -> Result<(), PbError> {
    DEMO_MODE.store(demo_mode, Relaxed);
    mode_change(GameModes::InGame)?;
    let options = OPTIONS.lock().map_err(|_| PbError::LockGeneric)?;
    if *options.music == true {
        midi::music_play();
    }
    let mut main_table = MAIN_TABLE.lock().map_err(|_| PbError::LockGeneric)?;
    let table = (*main_table).as_mut().unwrap();
    table.message(MessageCode::NEW_GAME, *options.players as f32);
    Ok(())
}

fn mode_change(mode: GameModes) -> Result<(), PbError> {
    let mut credits_active = CREDITS_ACTIVE.load(Relaxed);
    let box_guard = MISS_TEXT_BOX.lock().map_err(|_| PbError::LockGeneric)?;
    let miss_text_box = box_guard.as_ref();
    let mut idle_guard = IDLE_TIMER_MS.lock().map_err(|_| PbError::LockGeneric)?;

    if credits_active && miss_text_box.is_some() {
        miss_text_box.unwrap().clear(true);
    }
    credits_active = false;
    CREDITS_ACTIVE.store(credits_active, Relaxed);
    *idle_guard = 0.0;

    match mode {
        GameModes::InGame => {
            if (DEMO_MODE.load(Relaxed) == true) {
                LAUNCH_BALL_ENABLED.store(false, Relaxed);
                HIGH_SCORES_ENABLED.store(false, Relaxed);
                DEMO_ACTIVE.store(true, Relaxed);
                let mut main_table = MAIN_TABLE.lock()?;
                match main_table.as_mut() {
                    Some(table) => {
                        if table.demo.is_some() {
                            table.demo.as_mut().unwrap().active_flag = true;
                        }
                    }
                    None => {}
                }
            } else {
                LAUNCH_BALL_ENABLED.store(true, Relaxed);
                HIGH_SCORES_ENABLED.store(true, Relaxed);
                DEMO_ACTIVE.store(false, Relaxed);
                let mut main_table = MAIN_TABLE.lock()?;
                match main_table.as_mut() {
                    Some(mut table) => {
                        if table.demo.is_some() {
                            let table_demo = table.demo.as_mut().unwrap();
                            table_demo.active_flag = true;
                        }
                    }
                    None => {}
                }
            }
        }
        GameModes::GameOver => {
            LAUNCH_BALL_ENABLED.store(false, Relaxed);
            if DEMO_MODE.load(Relaxed) == false {
                HIGH_SCORES_ENABLED.store(true, Relaxed);
                DEMO_ACTIVE.store(false, Relaxed);
            }
            let main_table = MAIN_TABLE.lock()?;
            match (main_table.as_ref()) {
                Some(table) => {
                    if table.light_group.is_some() {
                        let light_group = table.light_group.as_ref().unwrap();
                        light_group.message(MessageCode::T_LIGHT_GROUP_GAME_OVER_ANIMATION, 1.4f32);
                    }
                }
                None => {}
            }
        }
    }
    let mut game_mode_grd = GAME_MODE.lock().map_err(|_| PbError::LockGeneric)?;
    *game_mode_grd = mode;

    Ok(())
}

pub(crate) fn uninit() {
    todo!()
}

pub fn ball_set(dx: f32, dy: f32) -> Result<(), PbError> {
    // dx and dy are normalized to window, ideally in [-1, 1]
    const SENSITIVITY: f32 = 7000.0;
    let mut table = MAIN_TABLE.lock()?;
    let table = (*table).as_mut().unwrap();
    for ball in &mut table.ball_list {
        if ball.base_component.active_flag.take() == true {
            ball.direction.x = dx * SENSITIVITY;
            ball.direction.y = dy * SENSITIVITY;

            // We're copying the ball to a mutable Vector2, so we mutate it and reassign back to the
            // original ball
            let ball_dir = &mut Vector2::from_vec3(ball.direction);
            ball.speed = normalize_2d(ball_dir);
            ball.direction = Vector3 {
                x: ball_dir.x,
                y: ball_dir.y,
                z: ball.direction.z,
            };
            ball.last_active_time = TIME_TICKS.load(SeqCst);
        }
    }

    Ok(())
}

pub(crate) fn frame(mut dt_milli_sec: f32) -> Result<(), PbError> {
    if dt_milli_sec > 100.0 {
        dt_milli_sec = 100.0;
    }
    if dt_milli_sec <= 0.0 {
        return Ok(());
    }

    if FULL_TILT_MODE.load(Relaxed) == true && DEMO_MODE.load(Relaxed) == false {
        let mut timer = IDLE_TIMER_MS.lock().unwrap();
        *timer += dt_milli_sec;
        if *timer >= 60000.0 && CREDITS_ACTIVE.load(Relaxed) == false {
            push_cheat("credits");
        }
    }

    let dt_sec = dt_milli_sec * 0.001f32;
    let mut time_next = *TIME_NEXT.lock().map_err(|_| PbError::LockGeneric)?;
    let mut time_now = *TIME_NOW.lock().map_err(|_| PbError::LockGeneric)?;
    time_next = time_now + dt_sec;
    timed_frame(dt_sec)?;

    Ok(())
}

fn timed_frame(time_delta: f32) -> Result<(), PbError> {
    let main_table = MAIN_TABLE.lock()?;
    let mut table = (*main_table).as_mut().unwrap();
    for ball in &mut table.ball_list {
        if ball.base_component.active_flag.take() == false
            || ball.has_group_flag
            || ball.collision_comp.is_some()
            || ball.speed >= 0.8f32
        {
            if ball.stuck_count > 0 {
                let dist: Vector2 = Vector2 {
                    x: ball.position.x - ball.prev_position.x,
                    y: ball.position.y - ball.prev_position.y,
                };
                let radius_x2 = ball.radius * 2.0f32;
                if radius_x2 * radius_x2 < maths::magnitude_sq(&dist) {
                    ball.stuck_count = 0;
                }
            }
            ball.last_active_time = TIME_TICKS.load(SeqCst);
        } else if (TIME_TICKS.load(SeqCst) - ball.last_active_time > 500) {
            let dist: Vector2 = Vector2 {
                x: ball.position.x - ball.prev_position.x,
                y: ball.position.y - ball.prev_position.y,
            };
            let radius_d2 = ball.radius / 2.0f32;
            ball.prev_position = ball.position;
            if radius_d2 * radius_d2 < maths::magnitude_sq(&dist) {
                ball.stuck_count = 0;
            } else {
                ball.stuck_count += 1;
            }
            control::unstuck_ball(ball, TIME_TICKS.load(SeqCst) - ball.last_active_time);
        }
    }

    let mut ball_steps: [i32; 20] = [0; 20];
    let mut ball_steps_distance: [f32; 20] = [0.0f32; 20];
    let mut max_step = -1;

    for index in 0..table.ball_list.len() {
        let ball = &mut table.ball_list[index];
        ball_steps[index] = -1;
        if ball.base_component.active_flag.take() != false {
            let mut vec_dst: Vector2 = Vector2 { x: 0.0, y: 0.0 };
            ball.time_delta = time_delta;
            if ball.time_delta > 0.01f32 && ball.speed < 0.8f32 {
                ball.time_delta = 0.01f32;
            }
            ball.collision_disabled_flag = false;
            if let Some(rc_ptr) = ball.collision_comp.as_ref().and_then(|weak| weak.upgrade()) {
                let mut collision_comp = rc_ptr.borrow_mut();
                collision_comp.field_effect(ball, &mut vec_dst);
            } else {
                // TODO: Implement this edge manager ig
                //t_table_layer::edge_manager.field_effects(ball, &mut vec_dst);
                vec_dst.x = ball.time_delta;
                vec_dst.y = ball.time_delta;
                ball.direction.x *= ball.speed;
                ball.direction.y *= ball.speed;
                // TODO: Boring shit
                //maths::vector_add(ball.direction, &vec_dst);
                // TODO: Boring shit
                // ball.speed = maths::normalize_2d(ball.direction);
                // TODO: Add static
                if ball.speed > BALL_MAX_SPEED.load(SeqCst) {
                    ball.speed = BALL_MAX_SPEED.load(SeqCst);
                }

                ball_steps_distance[index] = ball.speed * ball.time_delta;
                // TODO: Ball half radius static
                let ball_step =
                    (f32::ceil(ball_steps_distance[index] / BALL_HALF_RADIUS.load(SeqCst)) - 1.0)
                        as i32;
                ball_steps[index] = ball_step;
                if ball_step > max_step {
                    max_step = ball_step;
                }
            }
        }
    }

    Ok(())
}

fn push_cheat(name: &str) {
    for ch in name.as_bytes() {
        control::pbctrl_bdoor_controller(ch);
    }
}
