use crate::maths::{RayType, RectF};
use crate::t_ball::TBall;
use crate::t_collision_component::TCollisionComponent;
use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::rc::{Rc, Weak};

pub enum WallType {
    Circle = 0,
    Line = 1,
}

pub enum WallValue {
    //TODO: Define what will be possible here
    Empty,
}

pub struct TEdgeSegment<'a> {
    pub collision_component: Weak<RefCell<TCollisionComponent<'a>>>,
    pub active_flag: Rc<Cell<bool>>,
    pub processed_flag: bool,
    pub wall_value: WallValue,
    pub collision_group: u32,
}

pub trait TEdgeSegmentBehavior {
    fn edge_collision(&self, ball: &mut TBall, distance: f32);
    fn port_draw(&self);
    fn place_in_grid(&self, aabb: RectF);
    fn find_collision_distance(&self, ray: &RayType) -> f32;
}

impl TEdgeSegmentBehavior for TEdgeSegment<'_> {
    fn edge_collision(&self, ball: &mut TBall, distance: f32) {
        todo!()
    }

    // Original does nothing
    fn port_draw(&self) {}

    fn place_in_grid(&self, aabb: RectF) {
        todo!()
    }

    fn find_collision_distance(&self, ray: &RayType) -> f32 {
        todo!()
    }
}

impl<'a> TEdgeSegment<'a> {
    pub fn new(
        coll_comp: &Rc<RefCell<TCollisionComponent<'a>>>,
        active_flag: Rc<Cell<bool>>,
        collision_group: u32,
    ) -> Self {
        Self {
            collision_component: Rc::downgrade(coll_comp),
            active_flag,
            processed_flag: false,
            wall_value: WallValue::Empty,
            collision_group,
        }
    }

    pub fn install_wall(
        float_arr: &[f32],
        coll_comp: &Rc<RefCell<TCollisionComponent>>,
        active_flag: Rc<Cell<bool>>,
        collision_group: u32,
        offset: u32,
        wall_value: usize,
    ) -> Box<dyn TEdgeSegmentBehavior> {
        todo!()
    }
}
