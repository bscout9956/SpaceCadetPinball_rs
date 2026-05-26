use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::maths::*;
use crate::t_ball::TBall;
use crate::t_edge_segment::TEdgeSegment;

#[allow(non_snake_case)]
pub struct TCollisionComponent {
    edge_list: Vec<Rc<RefCell<TEdgeSegment>>>,
    elasticity: f32,
    smoothness: f32,
    boost: f32,
    threshold: f32,
    soft_hit_sound_id: i32,
    hard_hit_sound_id: i32,
    AABB: RectF,
}

trait TCollisionComponentBehavior {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &Vector2,
        distance: f32,
        edge: &TEdgeSegment,
    );
    fn field_effect(&mut self, ball: &TBall, vec_destination: &mut Vector2) -> i32;
    fn default_collision(
        &mut self,
        ball: &TBall,
        next_position: &mut Vector2,
        direction: &mut Vector2,
    ) -> bool;
}

impl TCollisionComponent {
    // TODO: Finish me
}

impl TCollisionComponentBehavior for TCollisionComponent {
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
        ball: &TBall,
        next_position: &mut Vector2,
        direction: &mut Vector2,
    ) -> bool {
        todo!()
    }
}
