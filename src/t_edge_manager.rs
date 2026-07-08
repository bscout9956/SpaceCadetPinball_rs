use crate::maths;
use crate::maths::{RayType, Vector2};
use crate::t_ball::TBall;
use crate::t_collision_component::ICollisionComponent;
use crate::t_edge_box::TEdgeBox;
use crate::t_edge_segment::IEdgeSegment;
use crate::t_line::EdgeSegmentError;
use anyhow::Result;
use std::cell::{Cell, RefCell, RefMut};
use std::rc::{Rc, Weak};

#[derive(Clone, Default)]
pub struct FieldEffectType {
    pub active_flag: Rc<Cell<bool>>,
    pub collision_group: i32,
    pub collision_component: Option<Weak<RefCell<dyn ICollisionComponent>>>,
}

pub struct TEdgeManager {
    pub advance_x: f32,
    pub advance_y: f32,
    pub max_box_x: i32,
    pub max_box_y: i32,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub width: f32,
    pub height: f32,
    pub box_array: Vec<TEdgeBox>,
    pub edge_array: Vec<Rc<RefCell<dyn IEdgeSegment>>>, // 1000
}
impl TEdgeManager {
    pub fn new(min_x: f32, min_y: f32, width: f32, height: f32) -> Self {
        Self {
            advance_x: width / 10f32,
            advance_y: height / 15f32,
            max_box_x: 10,
            max_box_y: 15,
            min_x,
            min_y,
            max_x: min_x + width,
            max_y: min_y + height,
            width,
            height,
            box_array: vec![TEdgeBox::default(); 10 * 15],
            edge_array: Vec::new(),
        }
    }

    pub fn box_x(&self, x: f32) -> i32 {
        i32::max(
            0,
            i32::min(
                f32::floor((x - self.min_x) / self.advance_x) as i32,
                self.max_box_x - 1,
            ),
        )
    }

    pub fn box_y(&self, y: f32) -> i32 {
        i32::max(
            0,
            i32::min(
                f32::floor((y - self.min_y) / self.advance_y) as i32,
                self.max_box_y - 1,
            ),
        )
    }

    pub fn increment_box_x(&self, x: i32) -> i32 {
        i32::min(x + 1, self.max_box_x - 1)
    }

    pub fn increment_box_y(&self, y: i32) -> i32 {
        i32::min(y + 1, self.max_box_y - 1)
    }

    pub fn add_edge_to_box(
        &mut self,
        x: i32,
        y: i32,
        edge: Option<Rc<RefCell<dyn IEdgeSegment>>>,
    ) -> Result<(), EdgeSegmentError> {
        if x < 0 || x >= self.max_box_x || y < 0 || y >= self.max_box_y {
            return Err(EdgeSegmentError::BoxCoordsOutOfRange(x, y));
        }

        let list = &mut self.box_array[(x + y * self.max_box_x) as usize].edge_list;

        if let Some(e_ref) = edge {
            let has_duplicate = list.iter().any(|e| Rc::ptr_eq(e, &e_ref));
            if has_duplicate {
                return Err(EdgeSegmentError::DuplicateInBox);
            }

            list.push(e_ref);
        }

        Ok(())
    }

    pub fn add_field_to_box(
        &mut self,
        x: i32,
        y: i32,
        field: Rc<RefCell<FieldEffectType>>,
    ) -> Result<(), EdgeSegmentError> {
        if x < 0 || x >= self.max_box_x || y < 0 || y >= self.max_box_y {
            return Err(EdgeSegmentError::BoxCoordsOutOfRange(x, y));
        }

        let list = &mut self.box_array[(x + y * self.max_box_x) as usize].field_list;

        let has_duplicate = list.iter().any(|e| Rc::ptr_eq(e, &field));
        if has_duplicate {
            return Err(EdgeSegmentError::DuplicateInBox);
        }

        list.push(field);
        Ok(())
    }

    pub fn find_collision_distance(
        &mut self,
        ray: &mut RayType,
        ball: &Rc<RefCell<TBall>>,
        edge: &mut Rc<RefCell<dyn IEdgeSegment>>,
    ) -> Result<f32> {
        let mut distance = 1000000000.0f32;
        let mut edge_index = 0;

        let x0 = ray.origin.x;
        let y0 = ray.origin.y;
        let x1 = ray.direction.x * ray.max_distance + ray.origin.x;
        let y1 = ray.direction.y * ray.max_distance + ray.origin.y;

        let x_box_0 = self.box_x(x0);
        let y_box_0 = self.box_y(y0);
        let x_box_1 = self.box_x(x1);
        let y_box_1 = self.box_y(y1);

        let dir_x = if x0 >= x1 { -1 } else { 1 };
        let dir_y = if y0 >= y1 { -1 } else { 1 };

        if y_box_0 == y_box_1 {
            if dir_x == 1 {
                for index_x in x_box_0..=x_box_1 {
                    edge_index = self.test_grid_box(
                        index_x,
                        y_box_0,
                        &mut distance,
                        edge,
                        ray,
                        ball,
                        edge_index,
                    )?;
                }
            } else {
                let mut index_x = x_box_0;
                while index_x >= x_box_1 {
                    edge_index = self.test_grid_box(
                        index_x,
                        y_box_0,
                        &mut distance,
                        edge,
                        ray,
                        ball,
                        edge_index,
                    )?;
                    index_x -= 1;
                }
            }
        } else if x_box_0 == x_box_1 {
            if dir_y == 1 {
                for index_y in y_box_0..=y_box_1 {
                    edge_index = self.test_grid_box(
                        x_box_0,
                        index_y,
                        &mut distance,
                        edge,
                        ray,
                        ball,
                        edge_index,
                    )?;
                }
            } else {
                let mut index_y = y_box_0;
                while index_y >= y_box_1 {
                    edge_index = self.test_grid_box(
                        x_box_0,
                        index_y,
                        &mut distance,
                        edge,
                        ray,
                        ball,
                        edge_index,
                    )?;

                    index_y -= 1;
                }
            }
        } else {
            let mut edge_index =
                self.test_grid_box(x_box_0, y_box_0, &mut distance, edge, ray, ball, 0);

            // Bresenham line formula: y = dYdX * (x - x0) + y0; dYdX = (y0 - y1) / (x0 - x1)
            let dy_dx = (y0 - y1) / (x0 - x1);
            // Precompute constant part: dYdX * (-x0) + y0
            let pre_comp = -x0 * dy_dx + y0;
            // X and Y indexes are offset by one when going forwards, not sure why
            let x_bias = if dir_x == 1 { 1 } else { 0 };
            let y_bias = if dir_y == 1 { 1 } else { 0 };

            let mut index_x = x_box_0;
            let mut index_y = y_box_0;
            while index_x != x_box_1 || index_y != y_box_1 {
                let y_discrete = (index_y + y_bias) as f32 * self.advance_y + self.min_y;
                let y_linear =
                    ((index_x + x_bias) as f32 * self.advance_x + self.min_x) * dy_dx + pre_comp;
                // :)
                let check = if dir_y == 1 {
                    y_linear >= y_discrete
                } else {
                    y_linear <= y_discrete
                };
                if check {
                    index_y += dir_y;
                    if y_linear == y_discrete {
                        // Advance indexY when discrete value is ahead/behind
                        // Advance indexX when discrete value matches linear value
                        index_x += dir_x;
                    }
                } else {
                    // Advance indexX otherwise
                    index_x += dir_x;
                }

                edge_index = self.test_grid_box(
                    index_x,
                    index_y,
                    &mut distance,
                    edge,
                    ray,
                    ball,
                    edge_index?,
                );
            }
        }

        for processed_edge in self.edge_array.iter() {
            processed_edge.borrow_mut().processed_flag().set(false);
        }
        self.edge_array.clear();

        Ok(distance)
    }

    fn test_grid_box(
        &mut self,
        x: i32,
        y: i32,
        dist_val: &mut f32,
        edge_dst: &mut Rc<RefCell<dyn IEdgeSegment>>,
        ray: &RayType,
        ball: &Rc<RefCell<TBall>>,
        mut edge_index: i32,
    ) -> Result<i32> {
        if x >= 0 && x < self.max_box_x && y >= 0 && y < self.max_box_y {
            let edge_list = self.box_array[(x + y * self.max_box_x) as usize]
                .edge_list
                .clone();
            for edge_entry in edge_list.iter().rev() {
                // TODO: As usize? sure this isn't going to break something?
                if !edge_entry.borrow().processed_flag().get()
                    && edge_entry.borrow().active_flag().get()
                    && (edge_entry.borrow().collision_group() as usize
                        & ray.collision_mask as usize)
                        != 0
                    && !ball.borrow().already_hit(edge_entry)
                {
                    edge_index += 1;
                    self.edge_array.push(edge_entry.clone());
                    edge_entry.borrow_mut().processed_flag().set(true);
                    let dist = edge_entry.borrow().find_collision_distance(ray);
                    if dist < *dist_val {
                        *dist_val = dist;
                        *edge_dst = edge_entry.clone();
                    }
                }
            }
        }
        Ok(edge_index)
    }

    pub(crate) fn normalize_box(&self, pt: Vector2) -> Vector2 {
        let mut x = pt.x.clamp(self.min_x, self.max_x) + f32::abs(self.min_x);
        let mut y = pt.y.clamp(self.min_y, self.max_y) + f32::abs(self.min_y);

        x /= self.width;
        y /= self.height;
        return Vector2 {
            x: 1.0 - x,
            y: 1.0 - y,
        };
    }

    pub(crate) fn field_effects(&mut self, ball: &mut RefMut<TBall>, dst_vec: &mut Vector2) {
        let mut vec: Vector2 = Vector2::default();

        let index = self.box_x(ball.position.x) + self.box_y(ball.position.y) * self.max_box_x;
        let edge_box = &self.box_array[index as usize];

        for field in edge_box.field_list.iter() {
            if field.borrow().active_flag.get()
                && ball.collision_mask > 0
                && field.borrow().collision_group > 0
            {
                let col_comp_opt = &field.borrow().collision_component;
                if let Some(col_comp) = col_comp_opt
                    && let Some(col_upgrade) = col_comp.upgrade()
                {
                    let mut col_borrow = col_upgrade.borrow_mut();
                    let f_e_result = col_borrow.field_effect(
                        &ball.position,
                        &ball.direction,
                        ball.speed,
                        &mut vec,
                    );
                    if f_e_result > 0 {
                        maths::vector_add(dst_vec, &vec);
                    }
                }
            }
        }
    }
}
