use crate::maths::Vector2;
use crate::score::ScoreStruct;
use crate::t_ball::TBall;
use crate::t_pinball_component::TPinballComponent;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

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
struct TLightGroup;
// End of temporary structs

pub struct TPinballTable {
    pub flipper_l: TFlipper,
    pub flipper_r: TFlipper,
    pub cur_score_struct: Option<ScoreStruct>,
    pub score_ballcount: Option<ScoreStruct>,
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
    pub active_flag: Rc<Cell<bool>>,
    pub replay_active_flag: i32,
    pub replay_timer: i32,
    pub unknown_p81: i32,
    pub unknown_p82: i32,
    pub tilt_lock_flag: bool,
    pub score_multipliers: Vec<i32>,
    t_pinball_component: Weak<RefCell<TPinballComponent>>,
}

impl TPinballTable<'_> {
    pub fn new() -> Self {
        let pinball_component = TPinballComponent::new(None, -1, false);
        let active_flag = Rc::new(Cell::new(true));

        let instance = Self {
            active_flag: active_flag.clone(),
            flipper_l: TFlipper,
            flipper_r: TFlipper,
            cur_score_struct: None,
            score_ballcount: None,
            score_player_number_1: None,
            cheats_used: 0,
            sound_index_1: 0,
            sound_index_2: 0,
            sound_index_3: 0,
            ball_in_drain_flag: 0,
            cur_score: 0,
            cur_score_e9: 0,
            light_show_timer: 0,
            end_game_timeout_timer: 0,
            tilt_timeout_timer: 0,
            player_scores: [
                ScoreStruct {
                    score: todo!(),
                    dirty_flag: todo!(),
                    background_bmp: todo!(),
                    offset_x: todo!(),
                    offset_y: todo!(),
                    width: todo!(),
                    height: todo!(),
                    char_bmp: todo!(),
                    msg_font: todo!(),
                },
                ScoreStruct {
                    score: todo!(),
                    dirty_flag: todo!(),
                    background_bmp: todo!(),
                    offset_x: todo!(),
                    offset_y: todo!(),
                    width: todo!(),
                    height: todo!(),
                    char_bmp: todo!(),
                    msg_font: todo!(),
                },
                ScoreStruct {
                    score: todo!(),
                    dirty_flag: todo!(),
                    background_bmp: todo!(),
                    offset_x: todo!(),
                    offset_y: todo!(),
                    width: todo!(),
                    height: todo!(),
                    char_bmp: todo!(),
                    msg_font: todo!(),
                },
                ScoreStruct {
                    score: todo!(),
                    dirty_flag: todo!(),
                    background_bmp: todo!(),
                    offset_x: todo!(),
                    offset_y: todo!(),
                    width: todo!(),
                    height: todo!(),
                    char_bmp: todo!(),
                    msg_font: todo!(),
                },
            ],
            player_count: 0,
            current_player: 0,
            plunger: TPlunger,
            drain: TDrain,
            demo: TDemo,
            x_offset: 0,
            y_offset: 0,
            width: 0,
            height: 0,
            component_list: vec![],
            ball_list: vec![],
            flipper_list: vec![],
            light_group: TLightGroup,
            gravity_dir_vect_mult: 0.0,
            gravity_angle_x: 0.0,
            gravity_angle_y: 0.0,
            collision_comp_offset: 0.0,
            // TODO What do? Verify me
            plunger_position: Vector2 { x: 0.0, y: 0.0 },
            score_multiplier: 0,
            score_added: 0,
            reflex_shot_score: 0,
            bonus_score: 0,
            bonus_score_flag: false,
            jackpot_score: 0,
            jackpot_score_flag: false,
            unknown_p71: 0,
            ball_count: 0,
            max_ball_count: 0,
            extra_balls: 0,
            multiball_count: 0,
            ball_locked_counter: 0,
            multiball_flag: false,
            unknown_p78: 0,
            replay_active_flag: 0,
            replay_timer: 0,
            unknown_p81: 0,
            unknown_p82: 0,
            tilt_lock_flag: false,
            score_multipliers: &[1, 1, 1, 1, 1], // TODO: change me
            t_pinball_component: Default::default(),
        };

        // TODO: Implement AddBall
        //let ball = AddBall(Vector2 { x: 0.0, y: 0.0 });
        //ball.disable();

        // TODO: Implement TTableLayer
        //TTableLayer::new(this);

        //let score1 = score::create("score1", render::background_bitmap);
        //Instance.cur_score_struct = Some(score1);
        //Instance.player_scores[0] = score1;
        // TODO: Continue implementing

        instance
    }
}
