use crate::context::component_context::ComponentContext;
use crate::control::ComponentControl;
use crate::maths::{RectF, Vector2, Vector3};
use crate::message_code::MessageCode;
use crate::render::RenderSpriteRef;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_edge_manager::TEdgeManager;
use crate::t_pinball_component::IPinballComponent;
use crate::t_pinball_table::TPinballTable;
use anyhow::Result;
use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

// TODO: Temporary struct until the real flipper port lands.
pub struct TFlipper {
    base: TCollisionComponent,
    edge_list: Vec<Rc<RefCell<dyn IEdgeSegment>>>,
    aabb: RectF,
}

impl Default for TFlipper {
    fn default() -> Self {
        Self {
            // TODO: Use new
            base: TCollisionComponent::default(),
            edge_list: Vec::new(),
            aabb: RectF {
                x_max: -10000.0,
                y_max: -10000.0,
                x_min: 10000.0,
                y_min: 10000.0,
            },
        }
    }
}

impl TFlipper {
    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<Self> {
        let base = TCollisionComponent::new(table, group_index, false, state)?.take();
        Ok(Self {
            base,
            edge_list: Vec::new(),
            aabb: RectF {
                x_max: -10000.0,
                y_max: -10000.0,
                x_min: 10000.0,
                y_min: 10000.0,
            },
        })
    }

    pub(crate) fn flipper_collision(&self, _p0: f32) {
        todo!()
    }
}

impl TFlipper {
    pub(crate) fn update_sprite(&self) {
        todo!()
    }
}

// TODO: Implement me asap
impl TFlipper {
    pub(crate) fn get_flipper_step_angle(&self, _p0: f32, _p1: &mut f32) -> f32 {
        todo!()
    }
}

impl IPinballComponent for TFlipper {
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
        self.base.port_draw()
    }

    fn message(
        &mut self,
        code: MessageCode,
        value: f32,
        component_context: &mut ComponentContext,
    ) -> Result<i32> {
        self.base.message(code, value, component_context)
    }

    fn set_active_flag(&mut self, active: bool) {
        self.base.set_active_flag(active);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_collision_component(&self) -> Option<&TCollisionComponent> {
        Some(&self.base)
    }

    fn set_control(&mut self, control: Option<Weak<RefCell<ComponentControl>>>) {
        self.base.set_control(control);
    }
}

impl ICollisionComponent for TFlipper {
    fn collision(
        &mut self,
        _ball: &mut TBall,
        _next_position: &Vector2,
        _direction: &mut Vector2,
        _distance: f32,
        _edge: &TEdgeSegment,
        _component_context: &mut ComponentContext,
    ) -> Result<()> {
        Ok(())
    }

    fn edge_list(&mut self) -> &mut Vec<Rc<RefCell<dyn IEdgeSegment>>> {
        &mut self.edge_list
    }

    fn field_effect(
        &mut self,
        _ball_position: &Vector3,
        _ball_direction: &Vector3,
        _ball_speed: f32,
        _vec_destination: &mut Vector2,
    ) -> i32 {
        0
    }

    fn set_AABB(&mut self, aabb: RectF) {
        self.aabb = aabb;
    }

    fn get_AABB(&self) -> Option<RectF> {
        Some(self.aabb.clone())
    }
}
