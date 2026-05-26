use crate::maths::*;
use crate::t_collision_component::TCollisionComponent;
use crate::t_edge_segment::TEdgeSegment;

#[derive(Copy, Clone)]
pub struct TBall<'a> {
    pub position: Vector3,
    pub prev_position: Vector3,
    pub direction: Vector3,
    pub speed: f32,
    pub ray_max_distance: f32,
    pub time_delta: f32,
    pub ramp_field_force: Vector2,
    pub collision_comp: &'a TCollisionComponent, // TODO: a'? Is that what we want?
    pub collision_mask: i32,
    pub collisions: &'a [TEdgeSegment; 16], // TODO: I guess it's correct?
    pub edge_collision_count: i32,
    pub edge_collision_reset_flag: bool,
    pub collision_offset: Vector3,
    pub collision_flag: i32,
    pub radius: f32,
    pub has_group_flag: bool,
    pub stuck_count: i32,
    pub last_active_time: i32,
    pub visual_z_array: &'a [f32; 50], // TODO: 'a again?
    pub collision_disabled_flag: bool,
}