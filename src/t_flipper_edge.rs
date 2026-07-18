use crate::context::component_context::ComponentContext;
use crate::maths::{self, *};
use crate::message_code::MessageCode;
use crate::state::pb_game_state::PbGameState;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_pinball_table::TPinballTable;
use anyhow::Result;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub struct TFlipperEdge {
    pub base: TEdgeSegment,
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
    pub(crate) fn new(
        col_comp: Option<Weak<RefCell<dyn ICollisionComponent>>>,
        active_flag: Rc<Cell<bool>>,
        col_group: u32,
        table: Option<Weak<RefCell<TPinballTable>>>,
        origin: Vector3,
        vec_t1: Vector3,
        vec_t2: Vector3,
        mut extend_speed: f32,
        mut retract_speed: f32,
        collision_mult: f32,
        elasticity: f32,
        smoothness: f32,
        state: &mut PinballState,
    ) -> Self {
        let base = TEdgeSegment::new(col_comp, active_flag, col_group);

        let mut vec_origin_t1 = Vector2 {
            x: vec_t1.x - origin.x,
            y: vec_t1.y - origin.y,
        };
        maths::normalize_2d(&mut vec_origin_t1);

        let mut vec_origin_t2 = Vector2 {
            x: vec_t2.x - origin.x,
            y: vec_t2.y - origin.y,
        };
        maths::normalize_2d(&mut vec_origin_t2);

        let mut col_comp_offset = 0.0;
        if let Some(t) = table.as_ref() {
            if let Some(table) = t.upgrade() {
                col_comp_offset = table.borrow().collision_comp_offset;
            }
        }

        let circle_base_radius = origin.z + col_comp_offset;
        let circle_t1_radius = vec_t1.z + col_comp_offset;
        let mut angle_max = f32::acos(maths::dot_product(&vec_origin_t1, &vec_origin_t2));
        if maths::cross(&vec_origin_t1, &vec_origin_t2) < 0.0 {
            angle_max = -angle_max;
        }

        // 3DPB and FT have different formats for flipper speed:
        // 3DPB: Time it takes for flipper to go from source to destination, in sec.
        // FT: Flipper movement speed, in radians per sec.
        if !state.pb_game_state.full_tilt_mode {
            let angle_max_abs = angle_max.abs();
            retract_speed = angle_max_abs / retract_speed;
            extend_speed = angle_max_abs / extend_speed;
        }

        let perp_origin_t1_cc = Vector2 {
            x: -vec_origin_t1.y,
            y: vec_origin_t1.x,
        };
        let perp_origin_t1_c = Vector2 {
            x: vec_origin_t1.y,
            y: -vec_origin_t1.x,
        };

        let dx = vec_t1.x - origin.x;
        let dy = vec_t1.y - origin.y;
        let distance_div = (dy * dy + dx * dx).sqrt() + col_comp_offset + vec_t1.z;

        let mut instance = Self {
            base,
            flipper_flag: MessageCode::T_FLIPPER_NULL,
            elasticity,
            smoothness,
            rot_origin: Vector2::from_vec3(origin),
            circle_base_radius,
            circle_base_radius_m_sq: circle_base_radius * 1.01 * (circle_base_radius * 1.01),
            circle_base_radius_sq: circle_base_radius * circle_base_radius,
            circle_t1_radius,
            circle_t1_radius_m_sq: circle_t1_radius * 1.01 * (circle_t1_radius * 1.01),
            circle_t1_radius_sq: circle_t1_radius * circle_t1_radius,
            angle_max,
            angle_remainder: 0.0,
            angle_dst: 0.0,
            current_angle: 0.0,
            collision_line_perp: Default::default(),
            a1_src: Vector2::new(
                perp_origin_t1_cc.x * circle_t1_radius + vec_t1.x,
                perp_origin_t1_cc.y * circle_t1_radius + vec_t1.y,
            ),
            a2_src: Vector2::new(
                perp_origin_t1_cc.x * circle_base_radius + origin.x,
                perp_origin_t1_cc.y * circle_base_radius + origin.y,
            ),
            b1_src: Vector2::new(
                perp_origin_t1_c.x * circle_base_radius + origin.x,
                perp_origin_t1_c.y * circle_base_radius + origin.y,
            ),
            b2_src: Vector2::new(
                perp_origin_t1_c.x * circle_t1_radius + vec_t1.x,
                perp_origin_t1_c.y * circle_t1_radius + vec_t1.y,
            ),
            collision_mult,
            t1_src: Vector2::from_vec3(vec_t1),
            t2_src: Vector2::from_vec3(vec_t2),
            distance_div,
            distance_div_sq: distance_div * distance_div,
            collision_direction: Default::default(),
            extend_speed,
            retract_speed,
            move_speed: 0.0,
            next_ball_position: Default::default(),
            a1: Default::default(),
            a2: Default::default(),
            b1: Default::default(),
            b2: Default::default(),
            t1: Default::default(),
            line_a: Default::default(),
            line_b: Default::default(),
            circle_base: Default::default(),
            circle_t1: Default::default(),
            inv_t1_radius: 1.0 / circle_t1_radius * 1.5,
            y_min: 0.0,
            y_max: 0.0,
            x_min: 0.0,
            x_max: 0.0,
            control_point_dirty_flag: false,
        };

        if instance.angle_max < 0.0 {
            std::mem::swap(&mut instance.a1_src, &mut instance.b1_src);
            std::mem::swap(&mut instance.a2_src, &mut instance.b2_src);
        }

        if instance.angle_max <= 0.0 {
            instance.extend_speed = -instance.extend_speed;
        } else {
            instance.retract_speed = -instance.retract_speed;
        }
        instance.set_control_points(instance.current_angle);

        instance
    }

    pub(crate) fn flipper_angle_delta(&self, time_delta: f32) -> f32 {
        if self.flipper_flag == MessageCode::T_FLIPPER_NULL {
            return 0.0f32;
        }

        let delta_angle = self.move_speed * time_delta;
        if f32::abs(delta_angle) > self.angle_remainder {
            return self.angle_dst - self.current_angle;
        }

        return delta_angle;
    }

    pub(crate) fn set_control_points(&mut self, angle: f32) {
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

    pub(crate) fn set_motion(&mut self, code: &mut MessageCode) -> MessageCode {
        match *code {
            MessageCode::T_FLIPPER_EXTEND => {
                self.angle_remainder = f32::abs(self.angle_max - self.current_angle);
                self.angle_dst = self.angle_max;
                self.move_speed = self.extend_speed;
            }
            MessageCode::T_FLIPPER_RETRACT => {
                self.angle_remainder = f32::abs(self.current_angle);
                self.angle_dst = 0.0;
                self.move_speed = self.retract_speed;
            }
            MessageCode::RESET => {
                self.angle_remainder = 0.0;
                self.angle_dst = 0.0;
            }
            _ => {}
        }

        if self.angle_remainder == 0.0 {
            *code = MessageCode::T_FLIPPER_NULL
        }

        self.flipper_flag = *code;
        *code
    }
}

impl IEdgeSegment for TFlipperEdge {
    fn active_flag(&self) -> Rc<Cell<bool>> {
        self.base.active_flag.clone()
    }

    fn edge_collision(
        &mut self,
        ball: &Rc<RefCell<TBall>>,
        _distance: f32,
        _ctx: &mut ComponentContext,
    ) -> Result<()> {
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
        let x_max = (self.t2_src.x + self.circle_t1_radius)
            .max(self.t1_src.x + self.circle_t1_radius)
            .max(self.rot_origin.x + self.circle_base_radius);
        let y_max = (self.t2_src.y + self.circle_t1_radius)
            .max(self.t1_src.y + self.circle_t1_radius)
            .max(self.rot_origin.y + self.circle_base_radius);
        let x_min = (self.t2_src.x - self.circle_t1_radius)
            .min(self.t1_src.x - self.circle_t1_radius)
            .min(self.rot_origin.x - self.circle_base_radius);
        let y_min = (self.t2_src.y - self.circle_t1_radius)
            .min(self.t1_src.y - self.circle_t1_radius)
            .min(self.rot_origin.y - self.circle_base_radius);

        aabb.merge(&RectF {
            x_max,
            y_max,
            x_min,
            y_min,
        });
        crate::t_table_layer::edges_insert_square(y_min, x_min, y_max, x_max, this_rc, None, state)
    }

    fn find_collision_distance(&self, ray: &RayType) -> f32 {
        todo!()
    }

    fn collision_group(&self) -> u32 {
        self.base.collision_group
    }

    fn processed_flag(&self) -> Rc<Cell<bool>> {
        self.base.processed_flag.clone()
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
