use crate::maths::{RayType, RectF, Vector2, Vector3};
use crate::t_ball::TBall;
use crate::t_collision_component::ICollisionComponent;
use crate::t_line::TLine;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::ptr::slice_from_raw_parts;
use std::rc::{Rc, Weak};

pub enum WallType {
    Circle = 0,
    Line = 1,
}

// TODO/VERIFY: I think this will always only hold a bunch of bytes, so Raw as u8
#[derive(Clone)]
pub enum WallValue {
    Empty,
    Raw(u8),
}

#[derive(Clone)]
pub struct TEdgeSegment {
    pub collision_component: Option<Weak<RefCell<dyn ICollisionComponent>>>,
    pub active_flag: Rc<Cell<bool>>,
    pub processed_flag: Rc<Cell<bool>>,
    pub wall_value: WallValue,
    pub collision_group: u32,
}

use crate::maths;
use crate::state::pb_game_state::PbGameState;
use crate::t_circle::TCircle;
use anyhow::{Context, Result};

pub trait IEdgeSegment {
    fn active_flag(&self) -> Rc<Cell<bool>>;
    fn edge_collision(&self, ball: &Rc<RefCell<TBall>>, distance: f32);
    fn port_draw(&self);
    fn place_in_grid(
        &self,
        aabb: &mut RectF,
        this_rc: Option<Rc<RefCell<dyn IEdgeSegment>>>,
        pb_game_state: &mut PbGameState,
    ) -> Result<()>;
    fn find_collision_distance(&self, ray: &RayType) -> f32;

    fn collision_group(&self) -> u32;

    fn processed_flag(&self) -> Rc<Cell<bool>>;

    fn as_any(&self) -> &dyn Any;
}

impl IEdgeSegment for TEdgeSegment {
    fn active_flag(&self) -> Rc<Cell<bool>> {
        self.active_flag.clone()
    }

    fn edge_collision(&self, ball: &Rc<RefCell<TBall>>, distance: f32) {
        todo!()
    }

    // Original does nothing
    fn port_draw(&self) {}

    fn place_in_grid(
        &self,
        aabb: &mut RectF,
        this_rc: Option<Rc<RefCell<dyn IEdgeSegment>>>,
        pb_game_state: &mut PbGameState,
    ) -> Result<()> {
        todo!()
    }

    fn find_collision_distance(&self, ray: &RayType) -> f32 {
        todo!()
    }

    fn collision_group(&self) -> u32 {
        self.collision_group
    }

    fn processed_flag(&self) -> Rc<Cell<bool>> {
        self.processed_flag.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
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
        coll_comp: Weak<RefCell<dyn ICollisionComponent>>,
        active_flag: &Rc<Cell<bool>>,
        collision_group: u32,
        offset: f32,
        wall_value: usize,
        state: &mut PbGameState,
    ) -> Result<Option<Rc<RefCell<dyn IEdgeSegment>>>> {
        let mut center: Vector2 = Default::default();
        let mut start: Vector2 = Default::default();
        let mut end: Vector2 = Default::default();
        let mut prev_center: Vector2 = Default::default();
        let mut edge: Option<Rc<RefCell<dyn IEdgeSegment>>> = Option::None;

        // Vec1 and Vec2, great names
        let mut vec1: Vector3 = Default::default();
        let mut vec2: Vector3 = Default::default();

        let mut dest_vec: Vector3 = Default::default();

        let wall_type = unsafe { (f32::floor(*float_arr) - 1.0f32) as i32 };
        let mut slice_length = 0usize;
        match wall_type {
            0 => {
                // Circle
                slice_length = 4;
            }
            1 => {
                slice_length = 5;
            }
            _ => {
                slice_length = std::cmp::max((2 * wall_type) + 1, 4) as usize;
            }
        }

        let float_arr_slice = slice_from_raw_parts(float_arr, slice_length);

        unsafe {
            match wall_type {
                0 => {
                    //Circle
                    center.x = (*float_arr_slice)[1];
                    center.y = (*float_arr_slice)[2];
                    let radius = offset + (*float_arr_slice)[3];
                    let circle = TCircle::new(
                        Some(coll_comp.clone()),
                        active_flag.clone(),
                        collision_group,
                        &center,
                        radius,
                    );
                    if let Some(circle_base) = circle.base.upgrade() {
                        circle_base.borrow_mut().wall_value = if wall_value == 0 {
                            WallValue::Raw(wall_value as u8)
                        } else {
                            WallValue::Empty
                        };
                    }
                    if let Some(comp) = &coll_comp.upgrade() {
                        let mut comp_borrow = comp.borrow_mut();
                        if let Some(mut aabb) = comp_borrow.get_AABB() {
                            circle
                                .place_in_grid(&mut aabb, None, state)
                                .context("Failed to place in grid")?;
                            comp_borrow.set_AABB(aabb);
                        }

                        let rc_circle = Rc::new(RefCell::new(circle));
                        comp_borrow.edge_list().push(rc_circle.clone());
                        edge = Some(rc_circle);
                    }
                }
                1 => {
                    // Line
                    start.x = (*float_arr_slice)[1];
                    start.y = (*float_arr_slice)[2];
                    end.x = (*float_arr_slice)[3];
                    end.y = (*float_arr_slice)[4];
                    let mut line = TLine::new_vec2(
                        Some(coll_comp.clone()),
                        active_flag.clone(),
                        collision_group,
                        &start,
                        &end,
                    );
                    line.base.wall_value = if wall_value == 0 {
                        WallValue::Raw(wall_value as u8)
                    } else {
                        WallValue::Empty
                    };
                    line.offset(offset);
                    if let Some(comp) = coll_comp.upgrade().as_mut() {
                        let mut comp_borrow = comp.borrow_mut();
                        if let Some(mut aabb) = comp_borrow.get_AABB() {
                            line.place_in_grid(&mut aabb, Option::None, state)
                                .context("Failed to place in grid")?;
                            comp_borrow.set_AABB(aabb);
                        }

                        let rc_line = Rc::new(RefCell::new(line));
                        comp_borrow.edge_list().push(rc_line.clone());
                        edge = Some(rc_line);
                    }
                }
                _ => {
                    let mut float_arr_ptr = float_arr.add(1);
                    prev_center.x = (*float_arr_slice)[(2 * wall_type - 1) as usize];
                    prev_center.y = (*float_arr_slice)[(2 * wall_type) as usize];

                    for index in 0..wall_type {
                        let float_arr_ptr_slice = slice_from_raw_parts(float_arr_ptr, slice_length);
                        let center_x_2: f32;
                        let center_y_2: f32;

                        if index >= wall_type - 1 {
                            center_x_2 = (*float_arr_slice)[1];
                            center_y_2 = (*float_arr_slice)[2];
                        } else {
                            center_x_2 = (*float_arr_ptr_slice)[2];
                            center_y_2 = (*float_arr_ptr_slice)[3];
                        }
                        let center_x_1: f32 = (*float_arr_ptr_slice)[0];
                        let center_y_1: f32 = (*float_arr_ptr_slice)[1];

                        center.x = center_x_1;
                        center.y = center_y_1;

                        if offset != 0.0f32 {
                            vec1.x = center_x_1 - prev_center.x;
                            vec1.y = center.y - prev_center.y;
                            vec2.x = center_x_2 - center_x_1;
                            vec2.y = center_y_2 - center.y;

                            maths::cross_vec3(&vec1, &vec2, &mut dest_vec);
                            if (dest_vec.z > 0.0 && offset > 0.0)
                                || (dest_vec.z < 0.0 && offset < 0.0)
                            {
                                let radius = offset * 1.001f32;
                                let circle = TCircle::new(
                                    Some(coll_comp.clone()),
                                    active_flag.clone(),
                                    collision_group,
                                    &center,
                                    radius,
                                );
                                if let Some(circle_base) = circle.base.upgrade().as_ref() {
                                    circle_base.borrow_mut().wall_value = if wall_value == 0 {
                                        WallValue::Raw(wall_value as u8)
                                    } else {
                                        WallValue::Empty
                                    };
                                }
                                if let Some(comp) = &coll_comp.upgrade() {
                                    let mut comp_borrow = comp.borrow_mut();
                                    if let Some(mut aabb) = comp_borrow.get_AABB() {
                                        circle
                                            .place_in_grid(&mut aabb, Option::None, state)
                                            .context("Failed to place in grid")?;
                                        comp_borrow.set_AABB(aabb);
                                    }

                                    let rc_circle = Rc::new(RefCell::new(circle));
                                    comp_borrow.edge_list().push(rc_circle.clone());
                                    edge = Some(rc_circle);
                                }
                            }
                        }

                        start.x = (*float_arr_ptr_slice)[0];
                        start.y = (*float_arr_ptr_slice)[1];
                        end.x = (*float_arr_ptr_slice)[2];
                        end.y = (*float_arr_ptr_slice)[3];
                        let mut line = TLine::new_vec2(
                            Some(coll_comp.clone()),
                            active_flag.clone(),
                            collision_group,
                            &start,
                            &end,
                        );
                        line.base.wall_value = if wall_value == 0 {
                            WallValue::Raw(wall_value as u8)
                        } else {
                            WallValue::Empty
                        };
                        line.offset(offset);
                        if let Some(comp) = coll_comp.upgrade().as_mut() {
                            let mut comp_borrow = comp.borrow_mut(); //RefCell already borrowed

                            if let Some(mut aabb) = comp_borrow.get_AABB() {
                                line.place_in_grid(&mut aabb, None, state)
                                    .context("Failed to place in grid")?;
                                comp_borrow.set_AABB(aabb);
                            }

                            let rc_line = Rc::new(RefCell::new(line));
                            comp_borrow.edge_list().push(rc_line.clone());
                            edge = Some(rc_line);
                            prev_center = center;
                        }

                        float_arr_ptr = float_arr_ptr.add(2);
                    }
                }
            }
        }
        Ok(edge)
    }
}
