use crate::maths::{CircleType, RayType, RectF, Vector2};
use crate::t_ball::TBall;
use crate::t_collision_component::ICollisionComponent;
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub struct TCircle {
    pub(crate) base: Weak<RefCell<TEdgeSegment>>,
    pub circle: CircleType,
}

use crate::state::pb_game_state::PbGameState;
use crate::t_table_layer;
use anyhow::{Context, Result};

impl IEdgeSegment for TCircle {
    fn active_flag(&self) -> Rc<Cell<bool>> {
        self.base.upgrade().unwrap().borrow().active_flag()
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
        let radius = f32::sqrt(self.circle.radius_sq);
        aabb.merge(&RectF {
            x_max: self.circle.center.x + radius,
            y_max: self.circle.center.y + radius,
            x_min: self.circle.center.x - radius,
            y_min: self.circle.center.y - radius,
        });

        t_table_layer::edges_insert_circle(&self.circle, this_rc, None, state)
            .context("Failed to insert circle")?;

        Ok(())
    }

    fn find_collision_distance(&self, ray: &RayType) -> f32 {
        todo!()
    }

    fn collision_group(&self) -> u32 {
        self.base.upgrade().unwrap().borrow().collision_group()
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

    fn as_tcircle(&self) -> Option<&TCircle> {
        Some(self)
    }
}

impl TCircle {
    pub fn new(
        coll_comp: Option<Weak<RefCell<dyn ICollisionComponent>>>,
        active_flag: Rc<Cell<bool>>,
        collision_group: u32,
        center: &Vector2,
        radius: f32,
    ) -> TCircle {
        let base = TEdgeSegment::new(coll_comp, active_flag, collision_group);
        let base_rc = Rc::new(RefCell::new(base));
        let downgraded_base = Rc::downgrade(&base_rc);

        let mut def_circle = CircleType::default();
        // TODO Field assignment outside of initializer with default
        def_circle.radius_sq = radius * radius;
        def_circle.center = *center;

        Self {
            base: downgraded_base,
            circle: def_circle,
        }
    }
}
