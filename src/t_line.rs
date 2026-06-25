use crate::maths::{LineType, RayType, RectF, Vector2};
use crate::t_ball::TBall;
use crate::t_collision_component::ICollisionComponent;
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::{maths, t_table_layer};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use thiserror::Error;

pub struct TLine {
    pub t_edge_segment: TEdgeSegment,
    pub line: LineType,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

#[derive(Error, Debug)]
pub enum EdgeSegmentError {
    #[error("Failed to lock Mutex")]
    LockError,
    #[error("No edge in option")]
    NoEdgeError,
    #[error("Box Coordinates out of range: `{0}` and `{1}`")]
    BoxCoordsOutOfRange(i32, i32),
    #[error("Duplicated inserted into box")]
    DuplicateInBox,
}

impl IEdgeSegment for TLine {
    fn edge_collision(&self, ball: &mut TBall, distance: f32) {
        todo!()
    }

    fn port_draw(&self) {
        todo!()
    }

    fn place_in_grid(
        &self,
        aabb: &mut RectF,
        this_rc: Rc<RefCell<dyn IEdgeSegment>>,
    ) -> Result<(), EdgeSegmentError> {
        aabb.merge(&RectF {
            x_max: f32::max(self.x0, self.x1),
            y_max: f32::max(self.y0, self.y1),
            x_min: f32::min(self.x0, self.x1),
            y_min: f32::min(self.y0, self.y0),
        });
        let mut edge_guard = t_table_layer::EDGE_MANAGER
            .lock()
            .map_err(|_| EdgeSegmentError::LockError)?;

        let edge_man = edge_guard.as_mut().ok_or(EdgeSegmentError::NoEdgeError)?;
        let mut x_box_0 = edge_man.box_x(self.x0);
        let mut y_box_0 = edge_man.box_y(self.y0);
        let x_box_1 = edge_man.box_x(self.x1);
        let y_box_1 = edge_man.box_y(self.y1);

        let dir_x = if self.x0 >= self.x1 { -1 } else { 1 };
        let dir_y = if self.y0 >= self.y1 { -1 } else { 1 };

        if y_box_0 == y_box_1 {
            if dir_x == 1 {
                while x_box_0 <= x_box_1 {
                    edge_man.add_edge_to_box(x_box_0, y_box_0, Rc::clone(&this_rc));
                    x_box_0 += 1
                }
            } else {
                while x_box_1 >= x_box_0 {
                    edge_man.add_edge_to_box(x_box_0, y_box_0, Rc::clone(&this_rc)); // :(
                    x_box_0 -= 1
                }
            }
        } else if x_box_0 == x_box_1 {
            if dir_y == 1 {
                while y_box_0 <= y_box_1 {
                    edge_man.add_edge_to_box(x_box_0, y_box_0, Rc::clone(&this_rc));
                    y_box_0 += 1
                }
            } else {
                while y_box_1 >= y_box_0 {
                    edge_man.add_edge_to_box(x_box_0, y_box_0, Rc::clone(&this_rc));
                    y_box_0 -= 1
                }
            }
        } else {
            edge_man.add_edge_to_box(x_box_0, y_box_0, Rc::clone(&this_rc));
            // Bresenham line formula: y = dYdX * (x - x0) + y0; dYdX = (y0 - y1) / (x0 - x1)
            let dy_dx = (self.y0 - self.y1) / (self.x0 - self.x1);
            // Precompute constant part: dYdX * (-x0) + y0
            let pre_comp = -self.x0 * dy_dx + self.y0;
            // X and Y indexes are offset by one when going forwards, not sure why
            let x_bias = if dir_x == 1 { 1 } else { 0 };
            let y_bias = if dir_y == 1 { 1 } else { 0 };

            let mut index_x = x_box_0;
            let mut index_y = y_box_0;

            while index_x != x_box_1 || index_y != y_box_1 {
                // Calculate y from indexY and from line formula
                let y_discrete = (index_y + y_bias) as f32 * edge_man.advance_y + edge_man.min_y;
                let y_linear = ((index_x + x_bias) as f32 * edge_man.advance_x + edge_man.min_x)
                    * dy_dx
                    + pre_comp;

                let matches_discrete = if dir_y == 1 {
                    y_linear >= y_discrete
                } else {
                    y_linear <= y_discrete
                };

                if matches_discrete {
                    index_y += dir_y;
                    if y_linear == y_discrete {
                        index_x += dir_x;
                    }
                } else {
                    // Advance index_x otherwise
                    index_x += dir_x;
                }
                edge_man.add_edge_to_box(index_x, index_y, Rc::clone(&this_rc));
            }
        }

        Ok(())
    }

    fn find_collision_distance(&self, ray: &RayType) -> f32 {
        todo!()
    }
}

impl TLine {
    pub fn new(
        coll_comp: Option<Weak<RefCell<dyn ICollisionComponent>>>,
        active_flag: Rc<Cell<bool>>,
        collision_group: u32,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
    ) -> TLine {
        let edge = TEdgeSegment::new(coll_comp, active_flag, collision_group);

        let mut instance = Self {
            t_edge_segment: edge,
            line: LineType::default(),
            x0,
            y0,
            x1,
            y1,
        };
        maths::line_init(&mut instance.line, x0, y0, x1, y1);
        instance
    }

    pub fn new_vec2(
        coll_comp: Option<Weak<RefCell<dyn ICollisionComponent>>>,
        active_flag: Rc<Cell<bool>>,
        collision_group: u32,
        start: &Vector2,
        end: &Vector2,
    ) -> TLine {
        let edge = TEdgeSegment::new(coll_comp, active_flag, collision_group);

        let mut instance = Self {
            t_edge_segment: edge,
            line: LineType::default(),
            x0: start.x,
            y0: start.y,
            x1: end.x,
            y1: end.y,
        };
        maths::line_init(
            &mut instance.line,
            instance.x0,
            instance.y0,
            instance.x1,
            instance.y1,
        );
        instance
    }

    pub fn offset(&mut self, offset: f32) {
        let off_x = offset * self.line.perpendicular.x;
        let off_y = offset * self.line.perpendicular.y;

        self.x0 += off_x;
        self.y0 += off_y;
        self.x1 += off_x;
        self.y1 += off_y;
        maths::line_init(&mut self.line, self.x0, self.y0, self.x1, self.y1);
    }
}
