use crate::context::component_context::ComponentContext;
use crate::maths::{RectF, Vector2, Vector3};
use crate::t_ball::TBall;
use crate::t_collision_component::ICollisionComponent;
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use anyhow::Result;
use std::cell::RefCell;
use std::rc::Rc;

// TODO: Temporary struct until the real flipper port lands.
pub struct TFlipper {
    edge_list: Vec<Rc<RefCell<dyn IEdgeSegment>>>,
    aabb: RectF,
}

impl Default for TFlipper {
    fn default() -> Self {
        Self {
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
    pub(crate) fn flipper_collision(&self, p0: f32) {
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
    pub(crate) fn get_flipper_step_angle(&self, p0: f32, p1: &mut f32) -> f32 {
        todo!()
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
