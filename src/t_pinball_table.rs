use crate::errors::LoaderError;
use crate::maths::{RectF, Vector2, Vector3};
use crate::message_code::MessageCode;
use crate::score::ScoreStruct;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_demo::TDemo;
use crate::t_light_group::{TLightGroup, TLightGroupError};
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_table_layer::{TTableLayer, TTableLayerError};
use crate::t_textbox::TTextBox;
use crate::{loader, pb, render, score, timer};
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
    pub plunger: Option<TPlunger>,
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
        for comp in self.component_list.iter() {
            let name = comp.borrow().group_name();
            if let Some(n) = name {
                if *n.borrow() == component_name {
                    return Some(comp.clone());
                }
            }
        }
        None
    }
}

impl TPinballTable {
    fn ball_count_in_rect_base(&self, rect: &RectF) -> i32 {
        let mut count = 0;
        for ball in self.ball_list.iter() {
            let ball_borrow = ball.borrow();
            if ball_borrow.base_component.active_flag.get()
                && ball_borrow.position.x >= rect.x_min
                && ball_borrow.position.y >= rect.y_min
                && ball_borrow.position.x <= rect.x_max
                && ball_borrow.position.y <= rect.y_max
            {
                count += 1;
            }
        }
        count
    }

    pub(crate) fn ball_count_in_rect(&self, pos: &Vector2, margin: f32) -> i32 {
        let rect = RectF {
            x_min: pos.x - margin,
            x_max: pos.x + margin,
            y_min: pos.y - margin,
            y_max: pos.y + margin,
        };

        self.ball_count_in_rect_base(&rect)
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

use crate::render::RenderSprite;
use crate::t_drain::TDrain;
use crate::t_flipper::TFlipper;
use crate::t_plunger::TPlunger;
use crate::t_wall::TWall;
use crate::utils::DrawContext;
use anyhow::{Context, Result, bail};

impl TPinballTable {
    pub fn new(state: &mut PinballState) -> Result<Rc<RefCell<Self>>> {
        let base = TPinballComponent::new(None, -1, false, state)?;
        base.active_flag.set(true);

        let mut short_arr_length = 0;

        let instance = Self {
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
            plunger: None,
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

        let table_rc = Rc::new(RefCell::new(instance));
        let table_weak = Some(Rc::downgrade(&table_rc));

        table_rc.borrow_mut().base.pinball_table = table_weak.clone();

        let ball = table_rc
            .borrow_mut()
            .add_ball(Vector2::default(), state)
            .context("Failed to add ball to table")?;
        if let Some(b) = ball {
            b.borrow_mut().disable();
        }

        TTableLayer::new(table_weak.clone(), state)?;

        let light_group = TLightGroup::new(table_weak.clone(), 0, state)?;

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

        let mut short_arr = loader::query_int_attribute(
            group_index_objects,
            1025,
            &mut short_arr_length,
            &mut state.loader_state,
        )?;

        if short_arr_length > 0 {
            for _i in 0..short_arr_length / 2 {
                unsafe {
                    let object_type = *short_arr;
                    let short_arr_p1 = short_arr.add(1);
                    let group_index = *short_arr_p1;
                    short_arr = short_arr_p1.add(1);
                    match object_type {
                        // TODO: Once you implement a new component, make sure that it is added to the table
                        // If it needs to modify something in the table, do it afterwards
                        1000 | 1010 => {
                            let wall = TWall::new(table_weak.clone(), group_index as i32, state)?;
                            table_rc
                                .borrow_mut()
                                .add_component(Rc::new(RefCell::new(wall)));
                        }
                        1001 => {
                            let plunger =
                                TPlunger::new(table_weak.clone(), group_index as i32, state)
                                    .context("Failed to create plunger")?;
                            table_rc
                                .borrow_mut()
                                .add_component(Rc::new(RefCell::new(plunger.clone())));
                            table_rc.borrow_mut().plunger_position = plunger.table_pos;
                            table_rc.borrow_mut().plunger = Some(plunger);
                        }
                        1033 => {
                            let bg_bmp = state.render_state.background_bitmap.clone();
                            let textbox = TTextBox::new(
                                table_weak.clone(),
                                group_index as i32,
                                state.fullscrn_state.resolution,
                                &bg_bmp,
                                state,
                            )?;
                            table_rc
                                .borrow_mut()
                                .add_component(Rc::new(RefCell::new(textbox.clone())));
                        }
                        _ => {
                            // TODO: Implement the rest of the objects
                            println!("Unimplemented/unknown object type: {}", object_type);
                        }
                    }
                }
            }
        }

        render::build_occlude_list(&mut state.render_state);

        // TODO: Potential bug where we modify the copy instead of the reference
        // TODO: Move stuff to game state?
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

        // TODO: Potential bug where we modify the copy instead of the reference
        // TODO: Move stuff to game state?
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
    ) -> Result<Option<Rc<RefCell<TBall>>>> {
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
                return Ok(None);
            }

            let table_weak = self.base.pinball_table.clone();

            let new_ball_rc =
                TBall::new(table_weak, -1, state).context("Failed to create TBall")?;
            self.collision_comp_offset = new_ball_rc.borrow().radius;

            self.add_component(new_ball_rc.clone());
            self.ball_list.push(new_ball_rc.clone());

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

        Ok(Some(ball_rc))
    }

    pub(crate) fn add_score(&mut self, score: i32, full_tilt_mode: bool) -> Result<i32> {
        if self.jackpot_score_flag {
            self.jackpot_score += score;
            let jackpot_limit = if !full_tilt_mode { 5000000 } else { 10000000 };
            if self.jackpot_score > jackpot_limit {
                self.jackpot_score = jackpot_limit;
            }
        }

        if self.bonus_score_flag {
            self.bonus_score += score;
            if self.bonus_score > 5000000 {
                self.bonus_score = 5000000;
            }
        }
        let added_score =
            self.score_added + score * self.score_multipliers[self.score_multiplier as usize];
        self.cur_score += added_score;
        if self.cur_score > 1000000000 {
            self.cur_score_e9 += 1;
            self.cur_score -= 1000000000;
        }

        score::set(&mut self.cur_score_struct, self.cur_score);
        Ok(added_score)
    }

    pub(crate) fn add_component<T: IPinballComponent + 'static>(
        &mut self,
        component_rc: Rc<RefCell<T>>,
    ) {
        let trait_rc: Rc<RefCell<dyn IPinballComponent>> = component_rc.clone();
        self.component_list.push(trait_rc);
    }
}

impl IPinballComponent for TPinballTable {
    fn render_sprite(&self) -> Option<&RenderSprite> {
        self.base.render_sprite.as_ref()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn group_name(&self) -> Option<Rc<RefCell<String>>> {
        self.base.group_name.clone()
    }

    fn group_index(&self) -> i32 {
        self.base.group_index
    }

    fn sprite_set(&mut self, index: i32) {
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

    fn message(&mut self, code: MessageCode, value: f32, draw_context: &mut DrawContext) -> i32 {
        let rc_text = String::new();

        // TODO: Is it just reset? If there's no more, use if let
        match code {
            MessageCode::RESET => {
                for component_rc in self.component_list.iter_mut() {
                    let mut component = component_rc.borrow_mut();
                    component.message(MessageCode::RESET, 0.0, draw_context);
                }
                if self.replay_timer > 0 {
                    timer::kill_id(self.replay_timer);
                }
                self.replay_timer = 0;
                if self.light_show_timer > 0 {
                    timer::kill_id(self.light_show_timer);
                    if let Some(lg) = &mut self.light_group {
                        lg.message(MessageCode::T_LIGHT_GROUP_RESET, 0.0, draw_context);
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

    fn set_active_flag(&mut self, active: bool) {
        self.base.active_flag.set(active);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
