use crate::maths::{self, *};
use crate::message_code::MessageCode;
use crate::state::pb_game_state::PbGameState;
use crate::t_ball::TBall;
use crate::t_edge_segment::IEdgeSegment;
use anyhow::Result;
use std::any::Any;
use std::cell::{Cell, RefCell};
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
impl TFlipperEdge {
    pub(crate) fn set_contol_points(&mut self, angle: f32) {
        let (sin, cos) = maths::sin_cos(angle);
        self.a1 = self.a1_src;
        self.a2 = self.a2_src;
        self.b1 = self.b1_src;
        self.b2 = self.b2_src;
        self.t1 = self.t1_src;
        maths::rotate_pt(&mut self.a1, sin, cos, &self.rot_origin);
        maths::rotate_pt(&mut self.a2, sin, cos, &self.rot_origin);
        maths::rotate_pt(&mut self.t1, sin, cos, &self.rot_origin);
        maths::rotate_pt(&mut self.b1, sin, cos, &self.rot_origin);
        maths::rotate_pt(&mut self.b2, sin, cos, &self.rot_origin);
        maths::line_init(&mut self.line_a, self.a1.x, self.a1.y, self.a2.x, self.a2.y);
        maths::line_init(&mut self.line_b, self.b1.x, self.b1.y, self.b2.x, self.b2.y);
        self.circle_base = CircleType {
            center: self.rot_origin,
            radius_sq: self.circle_base_radius_sq,
        };
        self.circle_t1 = CircleType {
            center: self.t1,
            radius_sq: self.circle_base_radius_sq,
        };
        self.control_point_dirty_flag = false;
    }
}

impl IEdgeSegment for TFlipperEdge {
    fn active_flag(&self) -> Rc<Cell<bool>> {
        todo!(); // MISSSING BASE COMP
    }

    fn edge_collision(&mut self, ball: &Rc<RefCell<TBall>>, distance: f32) {
        todo!()
    }

    fn port_draw(&self) {
        todo!()
    }

    fn place_in_grid(
        &self,
        aabb: &mut RectF,
        this_rc: Option<Rc<RefCell<dyn IEdgeSegment>>>,
        state: &mut PbGameState,
    ) -> Result<()> {
        todo!()
    }

    fn find_collision_distance(&self, ray: &RayType) -> f32 {
        todo!()
    }

    fn collision_group(&self) -> u32 {
        todo!()
    }

    fn processed_flag(&self) -> Rc<Cell<bool>> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_tflipper_edge(&self) -> Option<&TFlipperEdge> {
        Some(self)
    }

    fn as_tflipper_edge_mut(&mut self) -> Option<&mut TFlipperEdge> {
        Some(self)
    }
}
