use crate::maths::{Vector2, Vector3};
use crate::message_code::MessageCode;
use crate::score::ScoreStruct;
use crate::state::fullscrn_state::FullscrnState;
use crate::state::loader_state::LoaderState;
use crate::state::pb_game_state::PbGameState;
use crate::state::render_state::RenderState;
use crate::t_ball::TBall;
use crate::t_demo::TDemo;
use crate::t_light_group::TLightGroup;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::timer;
use std::cell::RefCell;
use std::rc::Rc;

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
pub struct TFlipper;

// TODO: Implement me asap

impl TFlipper {
    pub(crate) fn update_sprite(&self) {
        todo!()
    }
}

// TODO: Implement me asap
impl TFlipper {
    pub(crate) fn get_flipper_step_angle(&self, p0: f32, p1: &mut f32) -> f32 {
        todo!()
    }
}

struct TPlunger;
struct TDrain;

// End of temporary structs

pub struct TPinballTable {
    base: TPinballComponent,
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
    pub component_list: Vec<Rc<RefCell<dyn IPinballComponent>>>,
    pub ball_list: Vec<Rc<RefCell<TBall>>>,
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
    // TODO: Consider reducing to state
    pub fn new(
        pb_game_state: &mut PbGameState,
        render_state: &mut RenderState,
        fullscrn_state: &mut FullscrnState,
        loader_state: &mut LoaderState,
    ) -> Self {
        let short_arr_length: usize;
        let base = TPinballComponent::new(None, -1, false, loader_state);

        let mut instance = Self {
            base,
            flipper_l: TFlipper,
            flipper_r: TFlipper,
            cur_score_struct: None,
            score_ball_count: None,
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
            player_scores: std::array::from_fn(|_| ScoreStruct::default()),
            player_count: 0,
            current_player: 0,
            plunger: TPlunger,
            drain: TDrain,
            demo: None,
            x_offset: 0,
            y_offset: 0,
            width: 0,
            height: 0,
            component_list: vec![],
            ball_list: vec![],
            flipper_list: vec![],
            light_group: None,
            gravity_dir_vect_mult: 0.0,
            gravity_angle_x: 0.0,
            gravity_angle_y: 0.0,
            collision_comp_offset: 0.0,
            plunger_position: Default::default(),
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
            score_multipliers: vec![],
        };

        let ball = instance.add_ball(
            Vector2::default(),
            pb_game_state,
            render_state,
            fullscrn_state,
            loader_state,
        );
        match ball {
            Some(b) => {
                b.borrow_mut().disable();
            }
            None => {}
        }

        //TODO: pass self, let table_layer = TTableLayer::new()

        instance
    }

    pub fn message(&mut self, code: MessageCode, value: f32) -> i32 {
        let rc_text = String::new();

        match code {
            MessageCode::RESET => {
                for component_rc in self.component_list.iter_mut() {
                    let mut component = component_rc.borrow_mut();
                    component.message(MessageCode::RESET, 0.0);
                }
                if self.replay_timer > 0 {
                    timer::kill_id(self.replay_timer);
                }
                self.replay_timer = 0;
                if self.light_show_timer > 0 {
                    timer::kill_id(self.light_show_timer);
                    if let Some(lg) = &self.light_group {
                        lg.message(MessageCode::T_LIGHT_GROUP_RESET, 0.0);
                    }
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

    // TODO: Consider reducing to state
    fn add_ball(
        &mut self,
        position: Vector2,
        pb_game_state: &mut PbGameState,
        render_state: &mut RenderState,
        fullscrn_state: &mut FullscrnState,
        loader_state: &mut LoaderState,
    ) -> Option<Rc<RefCell<TBall>>> {
        let mut target_ball_rc: Option<Rc<RefCell<TBall>>> = None;

        for rc_ball in &self.ball_list {
            let cur_ball = rc_ball.borrow();

            if !cur_ball.base_component.active_flag.get() {
                drop(cur_ball);

                target_ball_rc = Some(Rc::clone(rc_ball));
                break;
            }
        }

        let ball_rc = if let Some(found_rc) = target_ball_rc {
            found_rc
        } else {
            if self.ball_list.len() >= 20 {
                return None;
            }

            let table_weak = self.base.pinball_table.clone();

            let new_ball_rc = TBall::new(
                table_weak,
                -1,
                pb_game_state,
                render_state,
                fullscrn_state.resolution,
                loader_state,
            );

            self.ball_list.push(Rc::clone(&new_ball_rc));
            new_ball_rc
        };

        {
            let mut ball = ball_rc.borrow_mut();

            ball.base_component.active_flag.set(true);
            ball.position.z = ball.radius;
            ball.direction = Vector3::default();
            ball.speed = 0.0f32;
            ball.time_delta = 0.0f32;
            ball.edge_collision_count = 0;
            ball.collision_flag = 0;
            ball.collision_mask = 1;
            ball.collision_comp = None;

            ball.position.x = position.x;
            ball.position.y = position.y;
            ball.prev_position = ball.position;
            ball.stuck_count = 0;
            ball.last_active_time = pb_game_state.time_ticks;
        }

        Some(ball_rc)
    }
}
