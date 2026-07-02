use crate::errors::PbError;
use crate::loader;
use crate::loader::VisualStruct;
use crate::maths::{RectF, Vector2, Vector2i};
use crate::message_code::MessageCode;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_pinball_table::TPinballTable;
use std::any::Any;
use std::cell::RefCell;
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
        time_ticks: usize,
    ) {
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

use crate::render::RenderSprite;
use crate::t_pinball_component::IPinballComponent;
use anyhow::Result;

impl TPlunger {
    pub(crate) fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<Self> {
        let mut visual: VisualStruct = VisualStruct::default();

        let base = TCollisionComponent::new(table, group_index, true, state)?;
        let mut owned_comp = Rc::unwrap_or_clone(base).into_inner();
        loader::query_visual(group_index, 0, &mut visual, state)?;
        owned_comp.hard_hit_sound_id = visual.kicker.hard_hit_sound_id;
        owned_comp.threshold = 1000000000.0;
        owned_comp.elasticity = 0.5;
        owned_comp.smoothness = 0.5;

        let mut instance = Self {
            base: owned_comp,
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

impl IPinballComponent for TPlunger {
    fn render_sprite(&self) -> Option<&RenderSprite> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn group_name(&self) -> Option<Rc<RefCell<String>>> {
        self.base.group_name.clone()
    }

    fn group_index(&self) -> i32 {
        todo!()
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

    fn message(&mut self, code: MessageCode, value: f32, time_ticks: usize) -> i32 {
        todo!()
    }

    fn set_active_flag(&mut self, active: bool) {
        todo!()
    }
}
