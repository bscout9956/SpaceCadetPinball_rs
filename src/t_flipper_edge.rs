use crate::maths::*;
use crate::message_code::MessageCode;
use crate::t_ball::TBall;
use crate::t_edge_segment::IEdgeSegment;
use anyhow::Result;
use std::cell::RefCell;
use std::rc::Rc;

pub struct TFlipperEdge {
    pub flipper_flag: MessageCode,
    pub elasticity: f32,
    pub smoothness: f32,
    pub rot_origin: Vector2,
    pub circle_base_radius: f32,
    pub circle_t1_radius: f32,
    pub circle_base_radius_sq: f32,
    pub circle_t1_radius_sq: f32,
    pub circle_base_radius_m_sq: f32,
    pub circle_t1_radius_m_sq: f32,
    pub angle_max: f32,
    pub angle_remainder: f32,
    pub angle_dst: f32,
    pub current_angle: f32,
    pub collision_line_perp: Vector2,
    pub a1_src: Vector2,
    pub a2_src: Vector2,
    pub b1_src: Vector2,
    pub b2_src: Vector2,
    pub collision_mult: f32,
    pub t1_src: Vector2,
    pub t2_src: Vector2,
    pub distance_div: f32,
    pub distance_div_sq: f32,
    pub collision_direction: Vector2,
    pub extend_speed: f32,
    pub retract_speed: f32,
    pub move_speed: f32,
    pub next_ball_position: Vector2,
    pub a1: Vector2,
    pub a2: Vector2,
    pub b1: Vector2,
    pub b2: Vector2,
    pub t1: Vector2,
    pub line_a: LineType,
    pub line_b: LineType,
    pub circle_base: CircleType,
    pub circle_t1: CircleType,
    pub inv_t1_radius: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub x_min: f32,
    pub x_max: f32,
    pub control_point_dirty_flag: bool,
}

impl IEdgeSegment for TFlipperEdge {
    fn edge_collision(&self, ball: &mut TBall, distance: f32) {
        todo!()
    }

    fn port_draw(&self) {
        todo!()
    }

    fn place_in_grid(
        &self,
        aabb: &mut RectF,
        this_rc: Option<Rc<RefCell<dyn IEdgeSegment>>>,
    ) -> Result<()> {
        todo!()
    }

    fn find_collision_distance(&self, ray: &RayType) -> f32 {
        todo!()
    }
}
