use crate::context::component_context::ComponentContext;
use crate::loader;
use crate::loader::VisualStruct;
use crate::maths::Vector2;
use crate::message_code::MessageCode;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_segment::TEdgeSegment;
use crate::t_pinball_component::IPinballComponent;
use crate::t_pinball_table::TPinballTable;
use anyhow::Result;
use std::cell::RefCell;
use std::rc::Weak;

#[derive(Default)]
pub struct TBumper {
    pub base: TCollisionComponent,
    pub bmp_index: i32,
    pub timer: i32,
    pub timer_time: f32,
    pub original_threshold: f32,
    pub sound_index_3: i32,
    pub sound_index_4: i32,
    pub player_data: [TBumperPlayerBackup; 4],
}

impl TBumper {
    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<Self> {
        let base = TCollisionComponent::new(table, group_index, true, state)?;
        let original_threshold = base.borrow().threshold;
        let mut visual = VisualStruct::default();

        loader::query_visual(group_index, 0, &mut visual, state)?;
        let timer_time = unsafe {
            *loader::query_float_attribute_ptr(group_index, 0, 407, &mut state.loader_state)?
        };
        Ok(Self {
            base: base.take(),
            bmp_index: 0,
            timer: 0,
            timer_time,
            original_threshold,
            sound_index_3: visual.sound_index_3,
            sound_index_4: visual.sound_index_4,
            player_data: std::array::from_fn(|_| TBumperPlayerBackup::default()),
        })
    }
}

#[derive(Default)]
pub struct TBumperPlayerBackup {
    pub message_field: i32,
    pub bmp_index: i32,
}

impl IPinballComponent for TBumper {
    fn as_tbumper(&self) -> Option<&TBumper> {
        Some(self)
    }

    fn message(
        &mut self,
        _code: MessageCode,
        _value: f32,
        _component_context: &mut ComponentContext,
    ) -> Result<i32> {
        todo!()
    }
}

impl ICollisionComponent for TBumper {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
        distance: f32,
        edge: &TEdgeSegment,
        component_context: &mut ComponentContext,
    ) -> Result<()> {
        todo!()
    }

    fn edge_list(&mut self) -> &mut Vec<Rc<RefCell<dyn IEdgeSegment>>> {
        self.base.edge_list()
    }

    fn set_AABB(&mut self, aabb: RectF) {
        self.base.set_AABB(aabb);
    }

    fn get_AABB(&self) -> Option<RectF> {
        self.base.get_AABB()
    }
}
