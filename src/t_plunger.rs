use crate::loader::VisualStruct;
use crate::maths::{RectF, Vector2};
use crate::message_code::MessageCode;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_pinball_table::TPinballTable;
use crate::{loader, pb};
use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct TPlunger {
    base: TCollisionComponent,
    pub pullback_timer_: i32,
    pub ballfeed_timer_: i32,
    pub max_pull_back: f32,
    pub pullback_increment: f32,
    pub pullback_delay: f32,
    pub sound_index_p1: i32,
    pub sound_index_p2: i32,
    pub pullback_started_flag: bool,
    pub some_counter: i32, // really?
    pub table_pos: Vector2,
}

impl ICollisionComponent for TPlunger {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
        distance: f32,
        edge: &TEdgeSegment,
        time_ticks: &mut ComponentContext,
    ) -> Result<()> {
        todo!()
    }

    fn edge_list(&mut self) -> &mut Vec<Rc<RefCell<dyn IEdgeSegment>>> {
        &mut self.base.edge_list
    }

    fn set_AABB(&mut self, aabb: RectF) {
        self.base.set_AABB(aabb);
    }

    fn get_AABB(&self) -> Option<RectF> {
        self.base.get_AABB()
    }
}

use crate::context::component_context::ComponentContext;
use crate::render::RenderSpriteRef;
use crate::t_edge_manager::TEdgeManager;
use crate::t_pinball_component::IPinballComponent;
use anyhow::{Result, bail};

impl TPlunger {
    pub(crate) fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<Self> {
        let mut visual: VisualStruct = VisualStruct::default();

        let base = TCollisionComponent::new(table, group_index, true, state)?;
        let mut base_mut = base.take();
        loader::query_visual(group_index, 0, &mut visual, state)?;
        base_mut.hard_hit_sound_id = visual.kicker.hard_hit_sound_id;
        base_mut.threshold = 1000000000.0;
        base_mut.elasticity = 0.5;
        base_mut.smoothness = 0.5;

        let mut instance = Self {
            base: base_mut,
            pullback_timer_: 0,
            ballfeed_timer_: 0,
            max_pull_back: 0.0,
            pullback_increment: 0.0,
            pullback_delay: 0.025,
            sound_index_p1: visual.sound_index_4,
            sound_index_p2: visual.sound_index_3,
            pullback_started_flag: false,
            some_counter: 0,
            table_pos: Default::default(),
        };

        if state.pb_game_state.full_tilt_mode {
            instance.max_pull_back = 50.0f32;
            instance.pullback_increment = 50.0f32 / instance.base.list_bitmap.len() as f32 * 8.0f32;
        } else {
            instance.max_pull_back = 100.0f32;
            instance.pullback_increment =
                f32::floor(100.0f32 / instance.base.list_bitmap.len() as f32 * 8.0f32);
        }

        let float_arr =
            loader::query_float_attribute_ptr(group_index, 0, 601, &mut state.loader_state)?;
        unsafe {
            let (pos_x, pos_y) = ((*float_arr), (*float_arr.add(1))); // Lazy
            instance.table_pos = Vector2::new(pos_x, pos_y);
        }

        Ok(instance)
    }
}

unsafe extern "C" fn released_timer(
    _timer_id: i32,
    caller: *mut c_void,
    _component_context: &mut ComponentContext,
) -> Result<()> {
    println!("TPlunger timer released");
    unsafe {
        let plunger = &mut *(caller as *mut TPlunger);
        plunger.base.threshold = 1000000000.0;
        plunger.base.boost = 0.0;
    }
    Ok(())
}

unsafe extern "C" fn pullback_timer(
    timer_id: i32,
    caller: *mut c_void,
    component_context: &mut ComponentContext,
) -> Result<()> {
    println!("Pullback timer!");
    unsafe {
        let plunger = &mut *(caller as *mut TPlunger);
        plunger.base.boost += plunger.pullback_increment;
        if plunger.base.boost <= plunger.max_pull_back {
            if plunger.some_counter > 0 {
                let timer_context = component_context as *mut ComponentContext;
                let timer_manager = &mut component_context.timer_manager;
                plunger.pullback_timer_ = timer_manager.borrow_mut().set(
                    plunger.pullback_delay / 4.0f32,
                    plunger as *mut TPlunger as *mut c_void,
                    pullback_timer,
                    &*timer_context,
                )?;
            } else {
                let timer_context = component_context as *mut ComponentContext;
                let timer_manager = &mut component_context.timer_manager;
                plunger.pullback_timer_ = timer_manager.borrow_mut().set(
                    plunger.pullback_delay,
                    plunger as *mut TPlunger as *mut c_void,
                    pullback_timer,
                    &*timer_context,
                )?;
            }
        } else {
            plunger.pullback_timer_ = 0;
            plunger.base.boost = plunger.max_pull_back;
        }

        let index = f32::floor(
            (plunger.base.list_bitmap.len() - 1) as f32 * plunger.base.boost
                / plunger.max_pull_back,
        ) as i32;
        plunger.sprite_set(index);
    }
    Ok(())
}

unsafe extern "C" fn ball_feed_timer(
    _timer_id: i32,
    caller: *mut c_void,
    component_context: &mut ComponentContext,
) -> Result<()> {
    unsafe {
        let plunger = caller as *mut TPlunger;
        (*plunger).message(MessageCode::PLUNGER_FEED_BALL, 0.0, component_context)?;
        Ok(())
    }
}

impl IPinballComponent for TPlunger {
    fn render_sprite(&self) -> Option<RenderSpriteRef> {
        self.base.render_sprite.clone()
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
        self.base.sprite_set(index);
    }

    fn get_coordinates(&self, edge_manager: &TEdgeManager) -> Vector2 {
        self.base.get_coordinates(edge_manager)
    }

    fn get_scoring(&self, index: u32) -> i32 {
        todo!()
    }

    fn port_draw(&self) {
        todo!()
    }

    fn message(
        &mut self,
        code: MessageCode,
        value: f32,
        component_context: &mut ComponentContext,
    ) -> Result<i32> {
        // TODO: All other messages lol
        match code {
            MessageCode::PLUNGER_INPUT_PRESSED => {
                let mut multiball_count_check = false;
                let mut tilt_lock_flag = false;
                if let Some(t) = self.base.pinball_table.as_ref() {
                    let t_up = t.upgrade();
                    if let Some(table) = t_up.as_ref() {
                        let table_ptr = table.as_ptr();
                        // TODO: Refactor
                        unsafe {
                            multiball_count_check = (*table_ptr).multiball_count > 0;
                            tilt_lock_flag = (*table_ptr).tilt_lock_flag;
                        }
                    }
                }
                if !self.pullback_started_flag
                    && (!component_context.full_tilt_mode
                        || multiball_count_check && !tilt_lock_flag)
                {
                    self.pullback_started_flag = true;
                    self.base.boost = 0.0;
                    self.base.threshold = 1000000000.0;
                    // TODO: loader::play_sound(hardhitsoundid, this, tplunger1);
                    unsafe {
                        pullback_timer(0, &raw mut *self as *mut c_void, component_context)?;
                    }
                }
            }
            MessageCode::PLUNGER_FEED_BALL => {
                if let Some(t) = self.base.pinball_table.as_ref() {
                    let t_up = t.upgrade();
                    if let Some(table) = t_up.as_ref() {
                        let table_ptr = table.as_ptr();
                        unsafe {
                            if (*table_ptr).ball_count_in_rect(
                                &(*table_ptr).plunger_position,
                                (*table_ptr).collision_comp_offset * 1.2f32,
                            ) > 0
                            {
                                component_context.set_timer(
                                    1.0,
                                    &raw mut *self as *mut c_void,
                                    ball_feed_timer,
                                )?;
                            } else {
                                let ball = (*table_ptr).add_ball(
                                    (*table_ptr).plunger_position, component_context.time_ticks
                                )?;
                                if ball.is_none() {
                                    bail!("Failed to create ball in plunger");
                                }
                                (*table_ptr).multiball_count += 1;
                                (*table_ptr).ball_in_drain_flag = 0;
                                // TODO: pb::tilt_no_more(table_ptr, component_context.);
                            }
                        }
                    }
                }
            }
            MessageCode::PLUNGER_START_FEED_TIMER => {
                self.ballfeed_timer_ = component_context.timer_manager.borrow_mut().set(
                    0.95999998,
                    self as *mut TPlunger as *mut c_void,
                    ball_feed_timer,
                    component_context,
                )?;
                // TODO: loader::play_sound(soundindexp1, this, tplunger2);
            }
            MessageCode::PLUNGER_LAUNCH_BALL => {
                self.pullback_started_flag = true;
                self.base.boost = self.max_pull_back;
                self.message(
                    MessageCode::PLUNGER_INPUT_RELEASED,
                    0.0f32,
                    component_context,
                )?;
            }
            MessageCode::RESUME
            | MessageCode::LOOSE_FOCUS
            | MessageCode::PLUNGER_INPUT_RELEASED => {
                if self.pullback_started_flag && self.some_counter == 0 {
                    self.pullback_started_flag = false;
                    self.base.threshold = 0.0;
                    if self.pullback_timer_ > 0 {
                        component_context
                            .timer_manager
                            .borrow_mut()
                            .kill_id(self.pullback_timer_)?;
                    }
                    self.pullback_timer_ = 0;
                    //TODO: loader::play_sound(soundindexp2, this, tplugner3);
                    self.sprite_set(0);
                    unsafe {
                        component_context.timer_manager.borrow_mut().set(
                            self.pullback_delay,
                            self as *mut TPlunger as *mut c_void,
                            released_timer,
                            component_context,
                        )?;
                    }
                }
            }
            MessageCode::RESET => {
                self.pullback_started_flag = false;
                self.base.boost = 0.0;
                self.base.threshold = 1000000000.0;
                self.some_counter = 0;

                component_context
                    .timer_manager
                    .borrow_mut()
                    .kill_id(self.ballfeed_timer_)?;
                component_context
                    .timer_manager
                    .borrow_mut()
                    .kill_id(self.pullback_timer_)?;
                println!("Time to kill the callback");
                component_context
                    .timer_manager
                    .borrow_mut()
                    .kill_callback(released_timer)?;

                self.sprite_set(0);
            }
            MessageCode(v) => {
                println!("Message code not yet implemented: {}", v);
            }
        }
        Ok(0)
    }

    fn set_active_flag(&mut self, active: bool) {
        self.base.active_flag.set(active);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_collision_component(&self) -> Option<&TCollisionComponent> {
        Some(&self.base)
    }
}