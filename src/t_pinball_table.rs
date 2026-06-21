use crate::errors::LoaderError;
use crate::maths::{Vector2, Vector2i, Vector3};
use crate::message_code::MessageCode;
use crate::score::ScoreStruct;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_demo::TDemo;
use crate::t_light_group::{TLightGroup, TLightGroupError};
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_table_layer::{TTableLayer, TTableLayerError};
use crate::t_textbox::TTextBox;
use crate::{control, loader, pb, render, score, timer};
use sdl2::sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_WARNING;
use std::any::Any;
use std::cell::RefCell;
use std::ffi::{CString, NulError};
use std::rc::Rc;
use thiserror::Error;

#[derive(Default)]
pub struct ScoreStructSuper {
    pub score_struct: ScoreStruct,
    pub score: i32,
    pub score_e9_part: i32,
    pub jackpot_score: i32,
    pub ball_count: i32,
    pub extra_balls: i32,
    pub ball_locked_counter: i32,
}

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
    pub player_scores: [ScoreStructSuper; 4],
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

impl TPinballTable {
    pub(crate) fn find_component(
        &self,
        group_index: i32,
    ) -> Option<Rc<RefCell<dyn IPinballComponent>>> {
        for cmp_ref in self.component_list.iter() {
            if cmp_ref.borrow().group_index() == group_index {
                return Some(cmp_ref.clone());
            }
        }

        let buffer = format!("{}", group_index);
        pb::show_message_box(
            SDL_MESSAGEBOX_WARNING,
            "Table cant find (lh):",
            &buffer,
            &None,
        )
        .unwrap();
        None
    }

    fn find_component_by_name(
        &self,
        component_name: &str,
    ) -> Option<Rc<RefCell<dyn IPinballComponent>>> {
        self.component_list
            .iter()
            .find(|cmp| cmp.borrow().group_name().as_deref() == Some(component_name))
            .cloned()
    }
}

impl TPinballTable {
    pub(crate) fn add_score(&self, val: i32) {
        todo!()
    }
}

impl TPinballTable {
    pub(crate) fn ball_count_in_rect(&self, p0: Vector2, p1: f32) -> i32 {
        todo!()
    }
}

unsafe impl Sync for TPinballTable {}

unsafe impl Send for TPinballTable {}

#[derive(Debug, Error)]
pub enum PinballTableError {
    #[error("Error loading data: `{0}`")]
    LoaderError(#[from] LoaderError),
    #[error("Error creating string: `{0}`")]
    NulError(#[from] NulError),
    #[error("Unable to find score")]
    NoScore,
    #[error(transparent)]
    TLightGroupError(#[from] TLightGroupError),
    #[error(transparent)]
    TTableLayerError(#[from] TTableLayerError),
}

use crate::t_drain::TDrain;
use crate::t_flipper::TFlipper;
use crate::t_plunger::TPlunger;
use anyhow::{Result, bail};

impl TPinballTable {
    pub fn new(state: &mut PinballState) -> Result<Rc<RefCell<Self>>> {
        let base = TPinballComponent::new(None, -1, false, &mut state.loader_state);
        let mut short_arr_length = 0;

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
            player_scores: std::array::from_fn(|_| ScoreStructSuper::default()),
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

        let ball = instance.add_ball(Vector2::default(), state);
        if let Some(b) = ball {
            b.borrow_mut().disable();
        }

        let table_rc = Rc::new(RefCell::new(instance));
        let table_weak = Some(Rc::downgrade(&table_rc));
        TTableLayer::new(table_weak.clone(), state)?;

        let light_group = TLightGroup::new(table_weak.clone(), 0, &mut state.loader_state)?;

        table_rc.borrow_mut().light_group = Some(light_group);

        let base_score = score::create(
            "score1",
            state.render_state.background_bitmap.clone(),
            state,
        );
        if let Some(sc) = base_score {
            table_rc.borrow_mut().cur_score_struct = Some(sc.clone());
            table_rc.borrow_mut().player_scores[0].score_struct = sc.clone();

            for score_index in 1..4 {
                table_rc.borrow_mut().player_scores[score_index].score_struct =
                    score::dup(Some(sc.clone()), score_index);
            }
        } else {
            bail!(PinballTableError::NoScore);
        }

        table_rc.borrow_mut().current_player = 0;
        table_rc.borrow_mut().max_ball_count = 3;
        table_rc.borrow_mut().score_ball_count = score::create(
            "ballcount1",
            state.render_state.background_bitmap.clone(),
            state,
        );
        table_rc.borrow_mut().score_player_number_1 = score::create(
            "player_number1",
            state.render_state.background_bitmap.clone(),
            state,
        );
        let table_str = CString::new("table_objects".to_string())?;
        let group_index_objects =
            loader::query_handle(table_str.as_ptr(), &mut state.loader_state)?;

        let short_arr = loader::query_int_attribute(
            group_index_objects,
            1025,
            &mut short_arr_length,
            &mut state.loader_state,
        )?;

        // TODO: Create all instances for all the objects of the table.
        // TODO: THIS IS A BIG UNDERTAKING!
        // if short_arr_length > 0 {
        //     for i in 0..short_arr_length / 2 {
        //         let object_type = *short_arr;
        //         let short_arr_p1 = short_arr.add(1);
        //         let group_index = *short_arr_p1;
        //         short_arr = short_arr_p1.add(1);
        //         match object_type {
        //             1000 | 1010 => {
        //                 let _ = TWall
        //             }
        //         }
        //     }
        // }

        render::build_occlude_list();

        state.pb_game_state.info_text_box = table_rc
            .borrow_mut()
            .find_component_by_name("info_text_box")
            .and_then(|rc_comp| {
                let borrowed_comp = rc_comp.borrow();

                borrowed_comp
                    .as_any()
                    .downcast_ref::<TTextBox>()
                    .map(|tbox| tbox.to_owned())
            });

        state.pb_game_state.mission_text_box = table_rc
            .borrow_mut()
            .find_component_by_name("mission_text_box")
            .and_then(|rc_comp| {
                let borrowed_comp = rc_comp.borrow();
                borrowed_comp
                    .as_any()
                    .downcast_ref::<TTextBox>()
                    .map(|tbox| tbox.to_owned())
            });

        // control::make_links(table_weak.clone(), &mut state.control_state);
        Ok(table_rc)
    }

    pub(crate) fn add_ball(
        &mut self,
        position: Vector2,
        state: &mut PinballState,
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

            let new_ball_rc = TBall::new(table_weak, -1, state);

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
            ball.last_active_time = state.pb_game_state.time_ticks;
        }

        Some(ball_rc)
    }
}

impl IPinballComponent for TPinballTable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn group_name(&self) -> Option<String> {
        if let Some(name) = self.base.group_name.as_ref() {
            let name_str = name.borrow().to_string();
            return Some(name_str);
        }
        None
    }

    fn group_index(&self) -> i32 {
        self.base.group_index
    }

    fn sprite_set(&mut self, index: i32) {
        todo!()
    }

    fn sprite_set_ball(&self, index: i32, pos: Vector2i, depth: f32) {
        todo!()
    }

    fn get_coordinates(&self) -> Vector2 {
        todo!()
    }

    fn get_scoring(&self, index: u32) -> i32 {
        todo!()
    }

    fn port_draw(&self) {
        todo!()
    }

    fn message(&mut self, code: MessageCode, value: f32) -> i32 {
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
                    if let Some(lg) = &mut self.light_group {
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
}
