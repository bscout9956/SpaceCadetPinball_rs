use crate::maths::Vector2;
use crate::score::ScoreStruct;
use crate::t_ball::TBall;

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
struct TDemo;
struct TPinballComponent;
struct TLightGroup;
// End of temporary structs

pub struct TPinballTable {
    pub flipper_l: TFlipper,
    pub flipper_r: TFlipper,
    pub cur_score_struct: ScoreStruct,
    pub score_ballcount: ScoreStruct,
    pub score_player_number_1: ScoreStruct,
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
    pub demo: TDemo,
    pub x_offset: i32,
    pub y_offset: i32,
    pub width: i32,
    pub height: i32,
    pub component_list: Vec<TPinballComponent>,
    pub ball_list: Vec<TBall>,
    pub flipper_list: Vec<TFlipper>,
    pub light_group: TLightGroup,
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
    pub replay_active_flag: i32,
    pub replay_timer: i32,
    pub unknown_p81: i32,
    pub unknown_p82: i32,
    pub tilt_lock_flag: i32,
    pub score_multipliers: &'static [i32; 5],
}
