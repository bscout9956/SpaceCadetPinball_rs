use crate::context::component_context::ComponentContext;
use crate::control::ComponentControl;
use crate::loader::VisualStruct;
use crate::maths::{RectF, Vector2};
use crate::message_code::MessageCode;
use crate::render::RenderSpriteRef;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_manager::TEdgeManager;
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_pinball_component::IPinballComponent;
use crate::t_pinball_table::TPinballTable;
use crate::{control, loader};
use anyhow::Result;
use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct TGate {
    pub base: TCollisionComponent,
    pub sound_index_3: i32,
    pub sound_index_4: i32,
}

impl TGate {
    pub(crate) fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<Self> {
        let mut visual = VisualStruct::default();
        let base = TCollisionComponent::new(table, group_index, true, state)?;
        base.borrow_mut().set_active_flag(true);
        base.borrow_mut().sprite_set(0);
        loader::query_visual(group_index, 0, &mut visual, state)?;
        let mut inst = Self {
            base: base.take(),
            sound_index_3: visual.sound_index_3,
            sound_index_4: visual.sound_index_4,
        };
        control::handler(MessageCode::RESET, Some(&mut inst));
        Ok(inst)
    }
}

impl ICollisionComponent for TGate {
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
        todo!()
    }

    fn set_AABB(&mut self, aabb: RectF) {
        self.base.set_AABB(aabb);
    }

    fn get_AABB(&self) -> Option<RectF> {
        self.base.get_AABB()
    }
}

impl IPinballComponent for TGate {
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
        self.base.get_scoring(index)
    }

    fn port_draw(&self) {
        self.base.port_draw();
    }

    fn message(
        &mut self,
        code: MessageCode,
        value: f32,
        ctx: &mut ComponentContext,
    ) -> Result<i32> {
        match code {
            MessageCode::T_GATE_DISABLE => {
                self.set_active_flag(false);
                self.sprite_set(-1);
                loader::play_sound(self.sound_index_3, Some(self), "TGate1", ctx);
            }
            MessageCode::RESET | MessageCode::T_GATE_ENABLE => {
                self.set_active_flag(true);
                self.sprite_set(0);
                if code == MessageCode::T_GATE_ENABLE {
                    loader::play_sound(self.sound_index_4, Some(self), "TGate2", ctx);
                }
            }
            _ => {}
        }

        control::handler(code, Some(self));
        Ok(0)
    }

    fn set_active_flag(&mut self, active: bool) {
        self.base.set_active_flag(active);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_control(&mut self, control: Option<Weak<RefCell<ComponentControl>>>) {
        self.base.set_control(control);
    }
}
