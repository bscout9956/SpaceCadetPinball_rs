use crate::gdrv::ColorRgba;
use crate::group_data::DatFile;
use crate::pb::GameModes;
use crate::t_edge_manager::TEdgeManager;
use crate::t_pinball_table::TPinballTable;
use crate::t_textbox::TTextBox;
use crate::timer::TimerManager;
use dear_imgui_rs::sys::ImU32;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

pub struct PbGameState {
    pub ball_max_speed: f32,
    pub ball_half_radius: f32,
    pub ball_to_ball_collision_distance: f32,
    pub cheat_mode: bool,
    pub credits_active: bool,
    pub demo_mode: bool,
    pub full_tilt_mode: bool,
    pub full_tilt_demo_mode: bool,
    pub main_table: Option<Rc<RefCell<TPinballTable>>>,
    pub time_ticks: usize,
    pub time_next: f32,
    pub time_now: f32,
    pub quick_flag: bool,
    pub game_mode: GameModes,
    pub record_table: Option<Arc<RwLock<DatFile>>>,
    pub dat_file_name: String,
    pub base_path: String,
    pub idle_timer_ms: f32,
    pub mission_text_box: Option<Rc<RefCell<TTextBox>>>,
    pub info_text_box: Option<Rc<RefCell<TTextBox>>>,
    pub text_box_color: ImU32,
    pub current_palette: [ColorRgba; 256],
    pub edge_manager: Option<TEdgeManager>,
    pub time_ticks_remainder: f32,
}

impl PbGameState {
    pub(crate) fn increment_table_balls(&self) {
        if let Some(table) = &self.main_table {
            table.borrow_mut().extra_balls += 1;
        }
    }
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
            mission_text_box: None,
            info_text_box: None,
            text_box_color: 0,
            current_palette: std::array::from_fn(|_| ColorRgba::black()),
            edge_manager: None,
            time_ticks_remainder: 0.0,
        }
    }
}

impl Default for PbGameState {
    fn default() -> Self {
        Self::new()
    }
}
