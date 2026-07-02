use crate::errors::{PbError, TranslationError};
use crate::gdrv::ColorRgba;
use crate::group_data::{EntryBuffer, FieldTypes};
use crate::high_score::{HighScore, HighScoreEntry};
use crate::maths::{RayType, Vector2, Vector3, normalize_2d};
use crate::message_code::MessageCode;
use crate::options::{GameBindings, GameInput, InputTypes};
use crate::state::high_score_state::HighScoreState;
use crate::state::main_state::MainState;
use crate::state::options_state::OptionsState;
use crate::state::pb_game_state::PbGameState;
use crate::state::pinball_state::PinballState;
use crate::state::render_state::RenderState;
use crate::t_ball::TBall;
use crate::t_collision_component::ICollisionComponent;
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_pinball_component::IPinballComponent;
use crate::t_pinball_table::TPinballTable;
use crate::translations::Msg;
use crate::{
    SdlWindowPtr, control, gdrv, handle_game_binding, high_score, loader, maths, midi, nudge,
    options, partman, proj, render, score, timer, translations,
};
use anyhow::{Context, Result, bail};
use rand::random;
use sdl2::sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_ERROR;
use sdl2::sys::{SDL_KeyCode, SDL_MessageBoxFlags, SDL_ShowSimpleMessageBox};
use std::cell::{Cell, RefCell, RefMut};
use std::ffi::{CStr, CString, c_char};
use std::fs::File;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(PartialEq, Eq, Ord, PartialOrd)]
pub enum GameModes {
    InGame = 1,
    GameOver = 2,
}

pub fn make_path_name(file_name: &str, base_path: &str) -> String {
    format!("{}{}", base_path, file_name)
}

pub fn get_rc_string(u_id: Msg) -> Result<&'static str, TranslationError> {
    translations::get_translation(u_id)
}

pub fn get_rc_string_cstring(u_id: Msg) -> Result<CString, TranslationError> {
    let string = translations::get_translation(u_id)?;
    Ok(CString::new(string)?)
}

pub fn show_message_box(
    flags: SDL_MessageBoxFlags,
    title: &str,
    message: &str,
    main_window: &Option<SdlWindowPtr>,
) -> Result<(), PbError> {
    if flags == SDL_MESSAGEBOX_ERROR {
        eprint!("BL error {}\n{}\n", title, message);
    } else {
        print!("BL error {}\n{}\n", title, message);
    }

    let title = CString::new(title)?;
    let title_cstr = title.as_c_str();
    let message = CString::new(message)?;
    let message_cstr = message.as_c_str();

    if let Some(window) = main_window.as_ref() {
        unsafe {
            SDL_ShowSimpleMessageBox(
                flags as u32,
                title_cstr.as_ptr(),
                message_cstr.as_ptr(),
                window.0,
            );
        }
    }

    Ok(())
}

pub fn show_message_box_cstr_message(
    flags: SDL_MessageBoxFlags,
    title: &str,
    message: *const c_char,
    main_window: &Option<SdlWindowPtr>,
) -> Result<()> {
    let message_str = unsafe { CStr::from_ptr(message).to_str()? };
    show_message_box(flags, title, message_str, main_window)
        .context("Failed to show message box")?;
    Ok(())
}

pub fn select_dat_file(
    data_search_paths: &[&str],
    options_state: &mut OptionsState,
    pb_game_state: &mut PbGameState,
) {
    pb_game_state.dat_file_name.clear();
    pb_game_state.full_tilt_mode = false;
    pb_game_state.full_tilt_demo_mode = false;

    let mut dat_file_names: [&str; 3] = ["CADET.DAT", "PINBALL.DAT", "DEMO.DAT"];

    if *options_state.options.prefer_3dpb_game_data {
        dat_file_names.swap(0, 1);
    }

    for path in data_search_paths {
        if path.is_empty() {
            continue;
        }

        pb_game_state.base_path = path.to_string();

        for dat_file_name in dat_file_names {
            let mut file_name = dat_file_name.to_string();
            for i in 0..2 {
                if i == 1 {
                    file_name = file_name.to_lowercase();
                }

                let dat_file_path = make_path_name(&file_name, &pb_game_state.base_path);
                if let Err(e) = File::open(&dat_file_path) {
                    println!("Error opening dat_file {}: {}", &dat_file_path, e);
                    continue;
                }
                pb_game_state.dat_file_name = file_name;

                update_full_tilt_mode(dat_file_name, pb_game_state);
                return;
            }
        }
    }
}

fn update_full_tilt_mode(dat_file_name: &str, pb_game_state: &mut PbGameState) {
    if dat_file_name == "CADET.DAT" {
        pb_game_state.full_tilt_mode = true;
    }
    if dat_file_name == "DEMO.DAT" {
        pb_game_state.full_tilt_mode = true;
        pb_game_state.full_tilt_demo_mode = true;
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

pub fn init(state: &mut PinballState) -> Result<bool> {
    let fullscrn_state = &mut state.fullscrn_state;

    let mut projection_matrix: [f32; 12] = [0.0; 12];

    if state.pb_game_state.dat_file_name.is_empty() {
        return Ok(false);
    }
    let data_file_path = make_path_name(
        &state.pb_game_state.dat_file_name,
        &state.pb_game_state.base_path,
    );

    println!("Loading records");
    let dat = partman::load_records(
        data_file_path,
        state.pb_game_state.full_tilt_mode,
        fullscrn_state,
    )?;
    let shared_dat = Arc::new(RwLock::new(dat));

    state.pb_game_state.record_table = Some(Arc::clone(&shared_dat));

    let use_bmp_font: i32 = get_rc_int(Msg::TextBoxUseBitmapFont)?;
    if use_bmp_font == 1 {
        score::load_msg_font(
            "pbmsg_ft",
            &mut state.pb_game_state.record_table,
            fullscrn_state,
            &mut state.score_state,
        )?;
    }

    if state.pb_game_state.record_table.is_none() {
        return Ok(true);
    }

    let mut palette_colors = Vec::new();
    if let Some(table_arc) = state.pb_game_state.record_table.as_ref() {
        let t = table_arc.read().unwrap();
        let plt = t.field_labeled("background", FieldTypes::Palette);
        if let Some(EntryBuffer::Raw(data)) = plt {
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
        }
    }

    if !palette_colors.is_empty() {
        gdrv::display_palette(Some(&palette_colors), &mut state.pb_game_state);
    }

    let (background_bmp, camera_info) = {
        let table_arc = state.pb_game_state.record_table.as_ref().unwrap();
        let t = table_arc
            .read()
            .map_err(|_| PbError::NoTable)
            .context("Failure to set set Table RwLock to read")?;

        let background_bmp = t
            .get_bitmap(t.record_labeled("background"), fullscrn_state.resolution)
            .to_owned();

        let camera_info_id = t.record_labeled("camera_info") + fullscrn_state.resolution;
        let camera_data = t.field(camera_info_id, FieldTypes::FloatArray).unwrap();

        let camera_info = if let EntryBuffer::Raw(float_data) = camera_data {
            read_camera_floats(float_data.as_slice())
        } else {
            Vec::new()
        };

        (background_bmp, camera_info)
    };

    let res_val = fullscrn_state.resolution;
    let res_info = &fullscrn_state.resolution_array[res_val as usize];

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

    println!("Time to init render");
    render::init(
        None,
        res_info.table_width,
        res_info.table_height,
        &mut state.main_state,
        &mut state.options_state,
        &mut state.render_state,
        &mut state.pb_game_state,
    )?;

    if let Some(dst) = state.render_state.v_screen.as_mut()
        && let Some(b_bmp) = background_bmp
    {
        gdrv::copy_bitmap(
            dst,
            b_bmp.width,
            b_bmp.height,
            b_bmp.x_position,
            b_bmp.y_position,
            &b_bmp,
            0,
            0,
        );
    }

    loader::load_from(shared_dat, &mut state.loader_state)?;

    println!("Changing to in-game");
    mode_change(
        GameModes::InGame,
        &mut state.main_state,
        &mut state.pb_game_state,
    )?;

    state.pb_game_state.time_ticks = 0;
    timer::init(150)?;
    println!("Init'ing score");
    score::init();

    println!("Creating table...");

    match TPinballTable::new(state) {
        Ok(table) => state.pb_game_state.main_table = Some(table),
        Err(e) => {
            eprintln!("Failed to create TPinballTable: {:?}", e);
            std::process::exit(-1);
        }
    }

    let table = state.pb_game_state.main_table.as_ref().unwrap().borrow();
    let ball = &table.ball_list[0].borrow();

    state.pb_game_state.ball_max_speed = ball.radius * 200.0f32;
    state.pb_game_state.ball_half_radius = ball.radius * 0.5f32;
    state.pb_game_state.ball_to_ball_collision_distance =
        ball.radius + state.pb_game_state.ball_half_radius * 2.0f32;

    let mut red = 255;
    let mut green = 255;
    let mut blue = 255;

    if let Ok(font_color) = get_rc_string(Msg::TextBoxColor) {
        let mut parts = font_color.split_whitespace().map(|s| s.parse::<i32>());
        if let (Some(Ok(r)), Some(Ok(g)), Some(Ok(b))) = (parts.next(), parts.next(), parts.next())
        {
            red = r;
            green = g;
            blue = b;
        }
    }

    state.pb_game_state.text_box_color =
        ((255u32) << 24) | ((blue as u32) << 16) | ((green as u32) << 8) | (red as u32);
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

pub fn reset_table(pb_game_state: &mut PbGameState) -> Result<(), PbError> {
    pb_game_state
        .main_table
        .as_ref()
        .unwrap()
        .borrow_mut()
        .message(MessageCode::RESET, 0.0, pb_game_state.time_ticks);
    Ok(())
}

pub fn first_time_setup(
    render_state: &mut RenderState,
    pb_game_state: &mut PbGameState,
) -> Result<()> {
    render::update(render_state, pb_game_state).context("Failed to update render state")?;
    Ok(())
}

pub(crate) fn toggle_demo(state: &mut PinballState) -> Result<()> {
    if state.pb_game_state.demo_mode {
        state.pb_game_state.demo_mode = false;
        match state.pb_game_state.main_table.as_mut() {
            Some(table) => table.borrow_mut().message(
                MessageCode::RESET,
                0.0f32,
                state.pb_game_state.time_ticks,
            ),
            None => bail!(PbError::NoTable),
        };

        mode_change(
            GameModes::GameOver,
            &mut state.main_state,
            &mut state.pb_game_state,
        )?;
        if let Some(mtb) = state.pb_game_state.mission_text_box.as_mut() {
            mtb.clear(false);
        }
        if let Some(mut itb) = state.pb_game_state.info_text_box.take() {
            itb.display(
                get_rc_string(Msg::STRING125)?,
                -1.0f32,
                state.pb_game_state.time_ticks,
                state.pb_game_state.full_tilt_mode,
                &mut state.render_state.v_screen,
                &state.render_state.background_bitmap,
                &state.pb_game_state.current_palette,
                None,
            )
            .context("Failed to obtain RC String when toggling demo")?;
            state.pb_game_state.mission_text_box = Some(itb);
        }
    } else {
        replay_level(true, state)?;
    }
    Ok(())
}

pub fn replay_level(demo_mode: bool, state: &mut PinballState) -> Result<(), PbError> {
    state.pb_game_state.demo_mode = demo_mode;
    mode_change(
        GameModes::InGame,
        &mut state.main_state,
        &mut state.pb_game_state,
    )?;
    if *state.options_state.options.music {
        midi::music_play();
    }

    match state.pb_game_state.main_table.as_ref() {
        None => {
            return Err(PbError::NoTable);
        }
        Some(t) => {
            t.borrow_mut().message(
                MessageCode::NEW_GAME,
                *state.options_state.options.players as f32,
                state.pb_game_state.time_ticks,
            );
        }
    }

    Ok(())
}

fn mode_change(
    mode: GameModes,
    main_state: &mut MainState,
    pb_game_state: &mut PbGameState,
) -> Result<(), PbError> {
    let miss_text_box = pb_game_state.mission_text_box.as_ref();

    if pb_game_state.credits_active
        && let Some(text_box) = miss_text_box
    {
        text_box.clear(true);
    }
    pb_game_state.credits_active = false;
    pb_game_state.idle_timer_ms = 0.0;

    match mode {
        GameModes::InGame => {
            if pb_game_state.demo_mode {
                main_state.launch_ball_enabled = false;
                main_state.high_scores_enabled = false;
                main_state.demo_active = true;
                if let Some(table) = pb_game_state.main_table.as_mut()
                    && let Some(table_demo) = table.borrow_mut().demo.as_mut()
                {
                    table_demo.active_flag = true;
                }
            } else {
                main_state.launch_ball_enabled = true;
                main_state.high_scores_enabled = false;
                main_state.demo_active = false;
                if let Some(table) = pb_game_state.main_table.as_mut()
                    && let Some(table_demo) = table.borrow_mut().demo.as_mut()
                {
                    table_demo.active_flag = true;
                }
            }
        }
        GameModes::GameOver => {
            main_state.launch_ball_enabled = false;
            if !pb_game_state.demo_mode {
                main_state.high_scores_enabled = true;
                main_state.demo_active = false;
            }
            if let Some(table) = pb_game_state.main_table.as_mut()
                && let Some(light_group) = table.borrow_mut().light_group.as_mut()
            {
                light_group.message(
                    MessageCode::T_LIGHT_GROUP_GAME_OVER_ANIMATION,
                    1.4f32,
                    pb_game_state.time_ticks,
                );
            }
        }
    }
    pb_game_state.game_mode = mode;

    Ok(())
}

pub(crate) fn uninit(state: &mut PinballState) -> Result<i32, PbError> {
    loader::unload(&mut state.loader_state, &mut state.sound_state)?;
    high_score::write(
        &mut state.high_score_state,
        &mut state.options_state.settings,
    );
    // state.pb_game_state.main_table = Rc
    timer::uninit();
    Ok(0)
}

pub fn ball_set(dx: f32, dy: f32, pb_game_state: &mut PbGameState) {
    // dx and dy are normalized to window, ideally in [-1, 1]
    const SENSITIVITY: f32 = 7000.0;
    let mut table = pb_game_state.main_table.as_ref().unwrap().borrow_mut(); // Lazy, if this caused you trouble then fix me
    for ball_rc in &mut table.ball_list {
        let mut ball = ball_rc.borrow_mut();
        if ball.base_component.active_flag.get() {
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
            ball.last_active_time = pb_game_state.time_ticks;
        }
    }
}

pub(crate) fn frame(mut dt_milli_sec: f32, state: &mut PinballState) -> Result<()> {
    if dt_milli_sec > 100.0 {
        dt_milli_sec = 100.0;
    }
    if dt_milli_sec <= 0.0 {
        return Ok(());
    }

    if state.pb_game_state.full_tilt_mode && !state.pb_game_state.demo_mode {
        state.pb_game_state.idle_timer_ms += dt_milli_sec;
        if state.pb_game_state.idle_timer_ms >= 60000.0 && !state.pb_game_state.credits_active {
            push_cheat("credits", state)?;
        }
    }

    let dt_sec = dt_milli_sec * 0.001f32;
    state.pb_game_state.time_next = state.pb_game_state.time_now + dt_sec;
    timed_frame(dt_sec, &mut state.pb_game_state)?;

    Ok(())
}

fn timed_frame(time_delta: f32, pb_game_state: &mut PbGameState) -> Result<(), PbError> {
    let mut table = match pb_game_state.main_table.as_ref() {
        Some(table) => table.borrow_mut(),
        None => return Err(PbError::NoTable),
    };
    for ball_rc in &mut table.ball_list {
        let mut ball = ball_rc.borrow_mut();
        if !ball.base_component.active_flag.take()
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
            ball.last_active_time = pb_game_state.time_ticks;
        } else if pb_game_state.time_ticks - ball.last_active_time > 500 {
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

            let ball_time = ball.last_active_time;
            control::unstuck_ball(&mut ball, pb_game_state.time_ticks - ball_time);
        }
    }

    let mut ball_steps: [i32; 20] = [0; 20];
    let mut ball_steps_distance: [f32; 20] = [0.0f32; 20];
    let mut max_step = -1;

    for index in 0..table.ball_list.len() {
        let ball = &mut table.ball_list[index].borrow_mut();
        ball_steps[index] = -1;
        if ball.base_component.active_flag.take() {
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
                // t_table_layer::edge_manager.field_effects(ball, &mut vec_dst);
                vec_dst.x = ball.time_delta;
                vec_dst.y = ball.time_delta;
                ball.direction.x *= ball.speed;
                ball.direction.y *= ball.speed;
                maths::vector_add_vec2_to_vec3(&mut ball.direction, &vec_dst);
                ball.speed = maths::normalize_3d(&mut ball.direction);
                if ball.speed > pb_game_state.ball_max_speed {
                    ball.speed = pb_game_state.ball_max_speed;
                }

                ball_steps_distance[index] = ball.speed * ball.time_delta;
                let ball_step =
                    (f32::ceil(ball_steps_distance[index] / pb_game_state.ball_half_radius) - 1.0)
                        as i32;
                ball_steps[index] = ball_step;
                if ball_step > max_step {
                    max_step = ball_step;
                }
            }
        }
    }

    let mut delta_angle: [f32; 4] = [0.0f32; 4];
    let mut flipper_steps: [i32; 4] = [0; 4];

    for index in 0..table.flipper_list.len() {
        let flip_step = (table.flipper_list[index]
            .get_flipper_step_angle(time_delta, &mut delta_angle[index])
            - 1.0) as i32;
        flipper_steps[index] = flip_step;
        if flip_step > max_step {
            max_step = flip_step;
        }
    }

    let mut ray = RayType::default();
    ray.min_distance = 0.002f32;

    for step in 0..=max_step {
        for ball_index in 0..table.ball_list.len() {
            let ball = &table.ball_list[ball_index];
            if !ball.borrow().collision_disabled_flag && ball_steps[ball_index] >= step {
                ray.collision_mask = ball.borrow().collision_mask;

                let mut distance_sum = 0.0f32;
                while distance_sum < pb_game_state.ball_half_radius {
                    ray.origin = Vector2::from_vec3(ball.borrow().position);
                    ray.direction = Vector2::from_vec3(ball.borrow().direction);
                    if ball_steps[ball_index] <= step {
                        ray.max_distance = ball_steps_distance[ball_index]
                            - ball_steps[ball_index] as f32 * pb_game_state.ball_half_radius;
                    } else {
                        ray.max_distance = pb_game_state.ball_half_radius;
                    }

                    let mut edge: Rc<RefCell<dyn IEdgeSegment>> = Rc::new(RefCell::new(
                        TEdgeSegment::new(None, Rc::new(Cell::new(false)), 0),
                    ));

                    let mut distance = 0.0f32;
                    if let Some(edge_man) = pb_game_state.edge_manager.as_mut() {
                        distance = edge_man.find_collision_distance(&mut ray, ball, &mut edge);
                    }
                    if distance > 0.0f32 {
                        distance = ball_to_ball_collision(
                            &ray,
                            ball,
                            &mut edge,
                            &mut distance,
                            &table,
                            pb_game_state.ball_to_ball_collision_distance,
                        );
                    }
                    if ball.borrow().edge_collision_reset_flag {
                        ball.borrow_mut().edge_collision_reset_flag = false;
                    } else {
                        ball.borrow_mut().edge_collision_count = 0;
                        ball.borrow_mut().edge_collision_reset_flag = true;
                    }

                    if distance >= 1e9f32 {
                        ball.borrow_mut().position.x += ray.max_distance * ray.direction.x;
                        ball.borrow_mut().position.y += ray.max_distance * ray.direction.y;
                        break;
                    }

                    edge.borrow().edge_collision(ball, distance);
                    if distance <= 0.0f32 || ball.borrow().collision_disabled_flag {
                        break;
                    }
                    distance_sum += distance;
                }
            }
        }

        for flip_index in 0..table.flipper_list.len() {
            if flipper_steps[flip_index] >= step {
                table.flipper_list[flip_index].flipper_collision(delta_angle[flip_index]);
            }
        }
    }

    for flipper in &table.flipper_list {
        flipper.update_sprite();
    }

    for ball_rc in table.ball_list.iter() {
        let mut ball = ball_rc.borrow_mut();
        if ball.base_component.active_flag.take() {
            ball.repaint();
        }
    }

    Ok(())
}

fn ball_to_ball_collision(
    ray: &RayType,
    ball: &Rc<RefCell<TBall>>,
    edge: &mut Rc<RefCell<dyn IEdgeSegment>>,
    collision_distance: &mut f32,
    table: &RefMut<TPinballTable>,
    ball_to_ball_collision_dist: f32,
) -> f32 {
    for cur_ball in table.ball_list.iter() {
        if cur_ball.borrow().active_flag().get()
            && !std::ptr::eq(cur_ball, ball)
            && (cur_ball.borrow().collision_mask & ball.borrow().collision_mask) != 0
            && f32::abs(cur_ball.borrow().position.x - ball.borrow().position.x)
                < ball_to_ball_collision_dist
            && f32::abs(cur_ball.borrow().position.y - ball.borrow().position.y)
                < ball_to_ball_collision_dist
        {
            let mut distance = cur_ball.borrow().find_collision_distance(ray);
            if distance < 1e9f32 {
                distance = f32::max(0.0f32, distance - 0.002f32);
                if distance < *collision_distance {
                    *collision_distance = distance;
                    *edge = cur_ball.clone();
                }
            }
        }
    }

    *collision_distance
}

pub(crate) fn push_cheat(name: &str, state: &mut PinballState) -> Result<()> {
    for ch in name.as_bytes() {
        control::pbctrl_bdoor_controller(*ch, state)?;
    }
    Ok(())
}

pub(crate) fn pause_continue(state: &mut PinballState) -> Result<()> {
    state.main_state.single_step ^= true;

    if let Some(text_box) = state.pb_game_state.info_text_box.as_mut() {
        text_box.clear(false);
    }

    if let Some(miss_text_box) = state.pb_game_state.mission_text_box.as_ref() {
        miss_text_box.clear(false);
    }
    if state.main_state.single_step {
        let mut table = match state.pb_game_state.main_table.as_ref() {
            Some(table) => table.borrow_mut(),
            None => bail!(PbError::NoTable),
        };
        table.message(
            MessageCode::PAUSE,
            state.pb_game_state.time_now,
            state.pb_game_state.time_ticks,
        );
    }
    let rc_string = get_rc_string(Msg::STRING123)?;

    let mut text_box = state
        .pb_game_state
        .info_text_box
        .take()
        .ok_or(PbError::NoTextBox)?;

    text_box
        .display(
            rc_string,
            -1.0f32,
            state.pb_game_state.time_ticks,
            state.pb_game_state.full_tilt_mode,
            &mut state.render_state.v_screen,
            &state.render_state.background_bitmap,
            &state.pb_game_state.current_palette,
            None,
        )
        .context("Failed to display textbox in pause_continue")?;

    state.pb_game_state.info_text_box = Some(text_box);

    Ok(())
}

pub(crate) fn input_up(input: GameInput, state: &mut PinballState) -> Result<()> {
    if state.pb_game_state.game_mode != GameModes::InGame
        || state.main_state.single_step
        || state.pb_game_state.demo_mode
    {
        return Ok(());
    }

    let bindings = options::map_game_input(input, &mut state.options_state);
    for binding in bindings {
        let mut table = match state.pb_game_state.main_table.as_ref() {
            None => bail!(PbError::NoTable),
            Some(t) => t.borrow_mut(),
        };
        match binding {
            GameBindings::LeftFlipper => {
                table.message(
                    MessageCode::LEFT_FLIPPER_INPUT_RELEASED,
                    state.pb_game_state.time_now,
                    state.pb_game_state.time_ticks,
                );
            }
            GameBindings::RightFlipper => {
                table.message(
                    MessageCode::RIGHT_FLIPPER_INPUT_RELEASED,
                    state.pb_game_state.time_now,
                    state.pb_game_state.time_ticks,
                );
            }
            GameBindings::Plunger => {
                table.message(
                    MessageCode::PLUNGER_INPUT_PRESSED,
                    state.pb_game_state.time_now,
                    state.pb_game_state.time_ticks,
                );
            }
            GameBindings::LeftTableBump => {
                if !table.tilt_lock_flag {
                    nudge::nudge_right();
                }
            }
            GameBindings::RightTableBump => {
                if !table.tilt_lock_flag {
                    nudge::nudge_left();
                }
            }
            GameBindings::BottomTableBump if !table.tilt_lock_flag => {
                nudge::nudge_up();
            }

            _ => {}
        }
    }

    if state.pb_game_state.cheat_mode && input.input_type == InputTypes::Keyboard {
        const F12: i32 = SDL_KeyCode::SDLK_F12 as i32;

        if input.value == 0x62 {
            // 'b' {
            let pos = Vector2 {
                x: 6.0f32,
                y: 7.0f32,
            };

            if let Some(table) = state.pb_game_state.main_table.as_ref() {
                let table_rc = table.clone();
                let col_comp_offset = table_rc.borrow().collision_comp_offset * 1.2f32;
                let ball_count = table_rc.borrow().ball_count_in_rect(&pos, col_comp_offset);

                if ball_count == 0 {
                    let was_added = table_rc
                        .borrow_mut()
                        .add_ball(pos, state)
                        .context("Failed to add ball to table")?;
                    if was_added.is_some() {
                        table_rc.borrow_mut().multiball_count += 1;
                    }
                }
            }
        } else {
            let table_rc = match state.pb_game_state.main_table.as_ref() {
                None => bail!(PbError::NoTable),
                Some(t) => t,
            };
            let mut table = table_rc.borrow_mut();
            match input.value {
                0x68 => {
                    let entry = HighScore {
                        name: get_rc_string(Msg::STRING127)?,
                        score: 1000000000,
                    };
                    high_score::show_and_set_high_score_dialog(
                        HighScoreEntry { entry, position: 1 },
                        &mut state.high_score_state,
                    )
                }
                0x72 => {
                    control::cheat_bump_rank();
                }
                0x73 => {
                    table.add_score(
                        (random::<f32>() * 1000000.0f32) as i32,
                        state.pb_game_state.full_tilt_mode,
                    )?;
                    return Ok(());
                }
                F12 => {
                    table.port_draw();
                }
                0x69 => {
                    if let Some(lg) = table.light_group.as_mut() {
                        lg.message(
                            MessageCode::T_LIGHT_FT_TMP_OVERRIDE_ON,
                            1.0f32,
                            state.pb_game_state.time_ticks,
                        );
                    }
                }
                0x70 => {
                    if let Some(lg) = table.light_group.as_mut() {
                        lg.message(
                            MessageCode::T_LIGHT_FT_TMP_OVERRIDE_OFF,
                            1.0f32,
                            state.pb_game_state.time_ticks,
                        );
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub(crate) fn launch_ball(state: &mut PinballState) -> Result<(), PbError> {
    if let Some(table) = state.pb_game_state.main_table.as_mut() {
        let mut table_borrow = table.borrow_mut();
        let plunger = table_borrow.plunger.as_mut().unwrap();
        plunger.message(MessageCode::PLUNGER_LAUNCH_BALL, 0.0f32)?;
    }

    Ok(())
}

pub(crate) fn high_scores(high_score_state: &mut HighScoreState) {
    high_score::show_high_score_dialog(high_score_state);
}

pub(crate) fn lose_focus(
    table_option: &mut Option<Rc<RefCell<TPinballTable>>>,
    time_now: f32,
    time_ticks: usize,
) -> Result<()> {
    if let Some(table) = table_option {
        table
            .borrow_mut()
            .message(MessageCode::LOOSE_FOCUS, time_now, time_ticks);
        Ok(())
    } else {
        bail!(PbError::NoTable);
    }
}

pub(crate) fn input_down(input: GameInput, state: &mut PinballState) -> Result<()> {
    if state.options_state.control_waiting_for_input.is_some() {
        options::input_down(input, &mut state.options_state);
        return Ok(());
    }

    let bindings = options::map_game_input(input, &mut state.options_state);
    for bind in &bindings {
        handle_game_binding(bind, true);
    }

    if state.pb_game_state.game_mode != GameModes::InGame
        || state.main_state.single_step
        || state.pb_game_state.demo_mode
    {
        return Ok(());
    }

    if state.pb_game_state.credits_active {
        state
            .pb_game_state
            .mission_text_box
            .as_ref()
            .unwrap()
            .clear(true);
    }
    state.pb_game_state.credits_active = false;
    state.pb_game_state.idle_timer_ms = 0.0f32;

    if input.input_type == InputTypes::Keyboard {
        control::pbctrl_bdoor_controller(input.value as u8, state)?;
    }

    // TODO: Don't forget the other bindings!
    for binding in &bindings {
        match binding {
            GameBindings::LeftFlipper => {}
            GameBindings::RightFlipper => {}
            GameBindings::Plunger => {
                if let Some(t) = state.pb_game_state.main_table.as_ref() {
                    t.borrow_mut().message(
                        MessageCode::PLUNGER_INPUT_PRESSED,
                        state.pb_game_state.time_now,
                        state.pb_game_state.time_ticks,
                    );
                }
            }
            GameBindings::LeftTableBump => {}
            GameBindings::RightTableBump => {}
            GameBindings::BottomTableBump => {}
            _ => {}
        }
    }

    Ok(())
}
