use crate::group_data::DatFile;
use crate::pb::GameModes;
use crate::t_pinball_table::TPinballTable;
use crate::t_textbox::TTextBox;
use std::sync::Arc;

pub struct PbGameState {
    pub ball_max_speed: f32,
    pub ball_half_radius: f32,
    pub ball_to_ball_collision_distance: f32,
    pub cheat_mode: bool,
    pub credits_active: bool,
    pub demo_mode: bool,
    pub full_tilt_mode: bool,
    pub full_tilt_demo_mode: bool,
    pub main_table: Option<TPinballTable>,
    pub time_ticks: usize,
    pub time_next: f32,
    pub time_now: f32,
    pub quick_flag: bool,
    pub game_mode: GameModes,
    pub record_table: Option<Arc<DatFile>>,
    pub dat_file_name: String,
    pub base_path: String,
    pub idle_timer_ms: f32,
    pub miss_text_box: Option<TTextBox>,
}

impl PbGameState {
    pub fn new() -> Self {
        Self {
            ball_max_speed: 0.0,
            ball_half_radius: 0.0,
            ball_to_ball_collision_distance: 0.0,
            full_tilt_mode: false,
            full_tilt_demo_mode: false,
            cheat_mode: false,
            credits_active: false,
            demo_mode: false,
            main_table: None,
            time_ticks: 0,
            time_next: 0.0,
            time_now: 0.0,
            quick_flag: false,
            game_mode: GameModes::GameOver,
            record_table: None,
            dat_file_name: String::new(),
            base_path: String::new(),
            idle_timer_ms: 0.0f32,
            miss_text_box: None,
        }
    }
}
