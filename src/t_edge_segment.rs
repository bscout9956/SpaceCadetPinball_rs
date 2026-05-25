use std::cell::Cell;
use std::ffi::c_void;
use std::rc::Rc;

pub enum WallType {
    Circle = 0,
    Line = 1,
}

pub enum WallValue {
    //TODO: Define what will be possible here
    Empty,
}

// TODO: Temporary
pub struct TCollisionComponent {}

// TODO: Temporary
pub struct TBall {}

// TODO: Temporary
pub struct RectF {}

// TODO: Temporary
pub struct RayType {}

pub struct TEdgeSegment<'a> {
    pub collision_component: &'a TCollisionComponent,
    pub active_flag: Rc<Cell<bool>>,
    pub processed_flag: bool,
    pub wall_value: WallValue,
    pub collision_group: u32,
}

pub trait TEdgeSegmentBehavior {
    fn edge_collision(&self, ball: TBall, distance: f32);
    fn port_draw(&self);
    fn place_in_grid(&self, aabb: RectF);
    fn find_collision_distance(&self, ray: &RayType) -> f32;
}

impl<'a> TEdgeSegmentBehavior for TEdgeSegment<'a> {
    fn edge_collision(&self, ball: TBall, distance: f32) {
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
        coll_comp: &'a TCollisionComponent,
        active_flag: Rc<Cell<bool>>,
        collision_group: u32,
    ) -> Self {
        Self {
            collision_component: coll_comp,
            active_flag,
            processed_flag: false,
            wall_value: WallValue::Empty,
            collision_group,
        }
    }

    pub fn install_wall(
        float_arr: &[f32],
        coll_comp: &'a TCollisionComponent,
        active_flag: Rc<Cell<bool>>,
        collision_group: u32,
        offset: u32,
        wall_value: usize,
    ) -> Box<dyn TEdgeSegmentBehavior + 'a> {
        todo!()
    }
}
