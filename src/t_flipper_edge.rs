use crate::maths::*;
use crate::t_edge_segment::{TBall, TEdgeSegmentBehavior};
use crate::t_pinball_component::MessageCode;

struct TFlipperEdge {
    flipper_flag: MessageCode,
    elasticity: f32,
    smoothness: f32,
    rot_origin: Vector2,
    circle_base_radius: f32,
    circle_t1_radius: f32,
    circle_base_radius_sq: f32,
    circle_t1_radius_sq: f32,
    circle_base_radius_m_sq: f32,
    circle_t1_radius_m_sq: f32,
    angle_max: f32,
    angle_remainder: f32,
    angle_dst: f32,
    current_angle: f32,
    collision_line_perp: Vector2,
    a1_src: Vector2,
    a2_src: Vector2,
    b1_src: Vector2,
    b2_src: Vector2,
    collision_mult: f32,
    t1_src: Vector2,
    t2_src: Vector2,
    distance_div:f32, distance_div_sq: f32,
    collision_direction: Vector2,
    extend_speed: f32,
    retract_speed: f32,
    move_speed: f32,
    next_ball_position: Vector2,
    a1: Vector2, a2: Vector2, b1: Vector2, b2: Vector2, t1: Vector2,
    line_a: LineType, line_b: LineType,
    circle_base: CircleType, circle_t1: CircleType,
    inv_t1_radius: f32,
    y_min: f32, y_max: f32, x_min: f32, x_max: f32,
    control_point_dirty_flag: bool,
}

impl TEdgeSegmentBehavior for TFlipperEdge {
    fn edge_collision(&self, ball: &mut TBall, distance: f32) {
        todo!()
    }

    fn port_draw(&self) {
        todo!()
    }

    fn place_in_grid(&self, aabb: crate::t_edge_segment::RectF) {
        todo!()
    }

    fn find_collision_distance(&self, ray: &crate::t_edge_segment::RayType) -> f32 {
        todo!()
    }
}