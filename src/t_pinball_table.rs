use crate::maths::Vector2;
use crate::score::ScoreStruct;
use crate::t_ball::TBall;
use crate::t_demo::TDemo;
use crate::t_pinball_component::{MessageCode, TPinballComponent, TPinballComponentBehavior};
use crate::{control, timer};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use crate::t_light_group::TLightGroup;

pub struct ScoreStructSuper {
    pub score_struct: ScoreStruct,
    pub score: i32,
    pub score_e9_part: i32,
    pub jackpot_score: i32,
    pub ball_count: i32,
    pub extra_balls: i32,
    pub ball_locked_counter: i32,
}

// TODO: Temporary structs here \/
struct TFlipper;
struct TPlunger;
struct TDrain;

// End of temporary structs

pub struct TPinballTable {
    pub flipper_l: TFlipper,
    pub flipper_r: TFlipper,
    pub cur_score_struct: Option<ScoreStruct>,
    pub score_ball_count: Option<ScoreStruct>,
    pub score_player_number_1: Option<ScoreStruct>,
    pub cheats_used: i32,
    pub sound_index_1: i32,
    pub sound_index_2: i32,
    pub sound_index_3: i32,
    pub ball_in_drain_flag: i32,
    pub cur_score: i32,
    pub cur_score_e9: i32,
    pub light_show_timer: i32,
    pub end_game_timeout_timer: i32,
    pub tilt_timeout_timer: i32,
    pub player_scores: [ScoreStruct; 4],
    pub player_count: i32,
    pub current_player: i32,
    pub plunger: TPlunger,
    pub drain: TDrain,
    pub demo: Option<TDemo>,
    pub x_offset: i32,
    pub y_offset: i32,
    pub width: i32,
    pub height: i32,
    pub component_list: Vec<Rc<RefCell<TPinballComponent>>>,
    pub ball_list: Vec<TBall>,
    pub flipper_list: Vec<TFlipper>,
    pub light_group: Option<TLightGroup>,
    pub gravity_dir_vect_mult: f32,
    pub gravity_angle_x: f32,
    pub gravity_angle_y: f32,
    pub collision_comp_offset: f32,
    pub plunger_position: Vector2,
    pub score_multiplier: i32,
    pub score_added: i32,
    pub reflex_shot_score: i32,
    pub bonus_score: i32,
    pub bonus_score_flag: bool,
    pub jackpot_score: i32,
    pub jackpot_score_flag: bool,
    pub unknown_p71: i32, // lol?
    pub ball_count: i32,
    pub max_ball_count: i32,
    pub extra_balls: i32,
    pub multiball_count: i32,
    pub ball_locked_counter: i32,
    pub multiball_flag: bool,
    pub unknown_p78: i32,
    pub active_flag: Rc<Cell<bool>>,
    pub replay_active_flag: i32,
    pub replay_timer: i32,
    pub unknown_p81: i32,
    pub unknown_p82: i32,
    pub tilt_lock_flag: bool,
    pub score_multipliers: Vec<i32>,
}

unsafe impl Sync for TPinballTable {}

unsafe impl Send for TPinballTable {}

impl TPinballTable {
    pub fn new() {
        todo!()
    }
    
    pub fn message(&mut self, code: MessageCode, value: f32) -> i32 {
        let rc_text = String::new();

        match code {
            MessageCode::RESET => {
                for component in self.component_list.iter_mut() {
                    component.message(MessageCode::RESET, 0.0);
                }
                if self.replay_timer > 0 {
                    timer::kill_id(self.replay_timer);
                }
                self.replay_timer = 0;
                if self.light_show_timer > 0 {
                    timer::kill_id(self.light_show_timer);
                    self.light_group.unwrap()
                        .message(MessageCode::T_LIGHT_GROUP_RESET, 0.0);
                }
                self.light_show_timer = 0;
                self.score_multiplier = 0;
                self.score_added = 0;
                self.reflex_shot_score = 0;
                self.bonus_score = 10000;
                self.bonus_score_flag = false;
                self.jackpot_score = 20000;
                self.jackpot_score_flag = false;
                self.unknown_p71 = 0;
                self.extra_balls = 0;
                self.multiball_count = 0;
                self.ball_locked_counter = 0;
                self.multiball_flag = false;
                self.unknown_p78 = 0;
                self.replay_active_flag = 0;
                self.replay_timer = 0;
                self.tilt_lock_flag = false;
            }
            _ => {}
        }
        // TODO: Implement me
        //control::table_control_handler(code);
        0
    }
}
