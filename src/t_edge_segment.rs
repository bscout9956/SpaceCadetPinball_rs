use crate::maths::{RayType, RectF};
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_pinball_component::IPinballComponent;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub enum WallType {
    Circle = 0,
    Line = 1,
}

pub enum WallValue {
    //TODO: Define what will be possible here
    Empty,
}

pub struct TEdgeSegment {
    pub collision_component: Option<Weak<RefCell<dyn ICollisionComponent>>>,
    pub active_flag: Rc<Cell<bool>>,
    pub processed_flag: Rc<Cell<bool>>,
    pub wall_value: WallValue,
    pub collision_group: u32,
}

pub trait TEdgeSegmentBehavior {
    fn edge_collision(&self, ball: &mut TBall, distance: f32);
    fn port_draw(&self);
    fn place_in_grid(&self, aabb: RectF);
    fn find_collision_distance(&self, ray: &RayType) -> f32;
}

impl TEdgeSegmentBehavior for TEdgeSegment {
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

impl TEdgeSegment {
    pub fn new(
        collision_component: Option<Weak<RefCell<dyn ICollisionComponent>>>,
        active_flag: Rc<Cell<bool>>,
        collision_group: u32,
    ) -> Self {
        Self {
            collision_component,
            active_flag,
            processed_flag: Rc::new(Cell::new(false)),
            wall_value: WallValue::Empty,
            collision_group,
        }
    }

    pub fn install_wall(
        float_arr: *const f32,
        coll_comp: Weak<RefCell<dyn IPinballComponent>>,
        active_flag: bool,
        collision_group: u32,
        offset: f32,
        wall_value: usize,
    ) -> Box<dyn TEdgeSegmentBehavior> {
        todo!()
    }
}
