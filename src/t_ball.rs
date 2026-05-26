use crate::maths::*;
use crate::t_collision_component::{TCollisionComponent, TCollisionComponentBehavior};
use crate::t_edge_segment::{TEdgeSegment, TEdgeSegmentBehavior};
use crate::t_pinball_table::TPinballTable;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub struct TBall {
    pub position: Vector3,
    pub prev_position: Vector3,
    pub direction: Vector3,
    pub speed: f32,
    pub ray_max_distance: f32,
    pub time_delta: f32,
    pub ramp_field_force: Vector2,
    pub collision_comp: Option<Weak<RefCell<TCollisionComponent>>>,
    pub collision_mask: i32,
    pub collisions: [Option<Weak<RefCell<TEdgeSegment>>>; 16],
    pub edge_collision_count: i32,
    pub edge_collision_reset_flag: bool,
    pub collision_offset: Vector3,
    pub collision_flag: i32,
    pub radius: f32,
    pub has_group_flag: bool,
    pub stuck_count: i32,
    pub last_active_time: i32,
    pub visual_z_array: [f32; 50],
    pub collision_disabled_flag: bool,
    t_edge_segment: Weak<RefCell<TEdgeSegment>>,
}

impl TBall {
    pub fn new(table: TPinballTable, group_index: i32) -> Self {
        let active_flag = Rc::new(Cell::new(true)); // TODO: default??

        let collision_comp = Rc::new(RefCell::new(TCollisionComponent::new(
            table,
            group_index,
            false,
        )));

        let edge_segment = TEdgeSegment::new(&collision_comp, Rc::clone(&active_flag), 0);

        Self {
            position: todo!(),
            prev_position: todo!(),
            direction: todo!(),
            speed: todo!(),
            ray_max_distance: todo!(),
            time_delta: todo!(),
            ramp_field_force: todo!(),
            collision_comp: todo!(),
            collision_mask: todo!(),
            collisions: todo!(),
            edge_collision_count: todo!(),
            edge_collision_reset_flag: todo!(),
            collision_offset: todo!(),
            collision_flag: todo!(),
            radius: todo!(),
            has_group_flag: todo!(),
            stuck_count: todo!(),
            last_active_time: todo!(),
            visual_z_array: todo!(),
            collision_disabled_flag: todo!(),
            t_edge_segment: todo!(),
        }
    }
}

impl TCollisionComponentBehavior for TBall {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &Vector2,
        distance: f32,
        edge: &TEdgeSegment,
    ) {
        todo!()
    }

    fn field_effect(&mut self, ball: &TBall, vec_destination: &mut Vector2) -> i32 {
        todo!()
    }

    fn default_collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
    ) -> bool {
        todo!()
    }
}

impl TEdgeSegmentBehavior for TBall {
    fn edge_collision(&self, ball: &mut TBall, distance: f32) {
        todo!()
    }

    fn port_draw(&self) {
        todo!()
    }

    fn place_in_grid(&self, aabb: RectF) {
        todo!()
    }

    fn find_collision_distance(&self, ray: &RayType) -> f32 {
        todo!()
    }
}
