use crate::t_collision_component::ICollisionComponent;
use crate::t_edge_box::TEdgeBox;
use crate::t_edge_segment::IEdgeSegment;
use crate::t_line::EdgeSegmentError;
use std::cell::{Cell, RefCell};
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
}

unsafe impl Send for TEdgeManager {}
unsafe impl Sync for TEdgeManager {}
