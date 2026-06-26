use crate::errors::LoaderError;
use crate::gdrv::GdrvBitmap8;
use anyhow::Result;

use crate::loader::{VisualStruct, query_float_attribute_ptr};
use crate::maths::{
    CircleType, MathsError, RayType, RectF, RectangleType, Vector2, f32_vec_to_vec2,
};
use crate::render::{RenderSprite, VisualTypes};
use crate::state::pb_game_state::PbGameState;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_manager::{FieldEffectType, TEdgeManager};
use crate::t_pinball_table::TPinballTable;
use crate::{loader, maths, proj, render};
use anyhow::{Context, bail};
use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;
use std::ffi::CString;
use std::ptr::slice_from_raw_parts;
use std::rc::{Rc, Weak};
use std::slice::from_raw_parts;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub struct TTableLayer {
    pub base_component: TCollisionComponent,
    pub vis_bmp: Option<Arc<GdrvBitmap8>>,
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
    pub gravity_dir_x: f32,
    pub gravity_dir_y: f32,
    pub gravity_mult: f32,
    pub field: FieldEffectType,
}

#[derive(Error, Debug)]
pub enum TTableLayerError {
    #[error(transparent)]
    LoaderError(#[from] LoaderError),
    #[error("Failed to unwrap or upgrade table")]
    InvalidTable,
    #[error("Failed to convert vector `{0}`")]
    VecConversionError(#[from] MathsError),
    #[error("Failed to lock Mutex")]
    LockError,
}

use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_line::TLine;

impl TTableLayer {
    pub fn field_effect(ball: Option<&mut TBall>, vec_dst: Option<&mut Vector2>) {
        todo!("I am never finished omg");
    }

    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        state: &mut PinballState,
    ) -> Result<Rc<RefCell<Self>>> {
        let mut visual = VisualStruct::default();
        let mut rect = RectangleType::default();

        let table_str = CString::new("table").context("Faield to create table string")?;
        let group_index = loader::query_handle(table_str.as_ptr(), &mut state.loader_state)?;
        loader::query_visual(group_index, 0, &mut visual, state)?;
        let sprite_data = visual.bitmap;

        /*Full tilt: proj center first value is offset by resolution*/
        let float_ptr = loader::query_float_attribute_ptr(
            group_index,
            0,
            700 + state.fullscrn_state.resolution,
            &mut state.loader_state,
        )?;
        let proj_center = slice_from_raw_parts(float_ptr, 2);
        unsafe {
            proj::recenter(&(*proj_center)[0], &(*proj_center)[1]);
        }
        render::set_background_zmap(sprite_data.zmap.clone(), 0, 0, &mut state.render_state);

        let bmp = &sprite_data.bmp;
        rect.x_position = 0;
        rect.y_position = 0;

        if let Some(bmp) = bmp {
            rect.width = bmp.width;
            rect.height = bmp.height;
        }

        RenderSprite::new(
            VisualTypes::Background,
            (*bmp).clone(),
            sprite_data.zmap,
            0,
            0,
            Some(rect),
            &mut state.render_state,
        );

        // TODO: Missing sound index1,2,3 from L38 in CPP

        let table_angle_array =
            query_float_attribute_ptr(group_index, 0, 305, &mut state.loader_state)?;
        if !table_angle_array.is_null() {
            if let Some(t) = table.as_ref().unwrap().upgrade() {
                let table_slice = unsafe { from_raw_parts(table_angle_array, 3) };
                t.borrow_mut().gravity_dir_vect_mult = table_slice[0];
                t.borrow_mut().gravity_angle_x = table_slice[1];
                t.borrow_mut().gravity_angle_y = table_slice[2];
            }
        } else {
            if let Some(t) = table.as_ref().unwrap().upgrade() {
                t.borrow_mut().gravity_dir_vect_mult = 25.0f32;
                t.borrow_mut().gravity_angle_x = 0.5f32;
                t.borrow_mut().gravity_angle_y = FRAC_PI_2; // 1.570796f in the original
            }
        }

        let gravity_dir_x;
        let gravity_dir_y;

        // TODO: Remove upgrades? Unsure
        if let Some(t) = table.as_ref().unwrap().upgrade() {
            gravity_dir_x = f32::cos(t.borrow().gravity_angle_y)
                * f32::sin(t.borrow().gravity_angle_x)
                * t.borrow().gravity_dir_vect_mult;
            gravity_dir_y = f32::sin(t.borrow().gravity_angle_y)
                * f32::sin(t.borrow().gravity_angle_x)
                * t.borrow().gravity_dir_vect_mult;

            if let Some(b) = bmp {
                t.borrow_mut().x_offset = b.x_position;
                t.borrow_mut().y_offset = b.y_position;
                t.borrow_mut().width = b.width;
            }
        } else {
            return bail!(TTableLayerError::InvalidTable);
        }

        let gravity_mult: f32;
        if !state.pb_game_state.full_tilt_mode && !state.pb_game_state.full_tilt_demo_mode {
            let angle_mult =
                loader::query_float_attribute_ptr(group_index, 0, 701, &mut state.loader_state)?;
            gravity_mult = unsafe { *angle_mult };
        } else {
            gravity_mult = 0.2f32;
        }

        let mut base = TCollisionComponent::new(table.clone(), -1, false, state);
        base.borrow_mut().threshold = visual.kicker.threshold;
        base.borrow_mut().boost = 15.0f32;

        let edge_points = f32_vec_to_vec2(&visual.float_arr)?; // throws error

        let instance = Self {
            base_component: base.take(),
            vis_bmp: (*bmp).clone(),
            x_min: f32::min(
                edge_points[0].x,
                f32::min(edge_points[1].x, edge_points[2].x),
            ),
            y_min: f32::min(
                edge_points[0].y,
                f32::min(edge_points[1].y, edge_points[2].y),
            ),
            x_max: f32::max(
                edge_points[0].x,
                f32::max(edge_points[1].x, edge_points[2].x),
            ),
            y_max: f32::max(
                edge_points[0].y,
                f32::max(edge_points[1].y, edge_points[2].y),
            ),
            gravity_dir_x,
            gravity_dir_y,
            gravity_mult,
            field: Default::default(),
        };

        let height = instance.y_max - instance.y_min;
        let width = instance.x_max - instance.x_min;

        {
            state.pb_game_state.edge_manager = Some(TEdgeManager::new(
                instance.x_min,
                instance.y_min,
                width,
                height,
            ));
        }

        let rc_this = Rc::new(RefCell::new(instance));

        for i in 0..visual.float_arr_count as usize {
            let mut this_mut = rc_this.borrow_mut();
            let flag = this_mut.base_component.active_flag.clone();
            let weak_this: Weak<RefCell<dyn ICollisionComponent>> = Rc::downgrade(&rc_this) as _;
            let line = TLine::new(
                Some(weak_this),
                flag,
                visual.collision_group as u32,
                edge_points[i + 1].x,
                edge_points[i + 1].y,
                edge_points[i].x,
                edge_points[i].y,
            );

            line.place_in_grid(
                &mut this_mut.base_component.aabb,
                Option::None,
                &mut state.pb_game_state,
            )
            .context("Failed to place line in grid")?;

            let line_rc = Rc::new(RefCell::new(line));
            this_mut.base_component.edge_list.push(line_rc);
        }

        {
            let mut rc_borrow = rc_this.borrow_mut();
            rc_borrow.field.collision_group = -1;
            rc_borrow.field.active_flag = rc_borrow.base_component.active_flag.clone();
            let weak_this: Weak<RefCell<dyn ICollisionComponent>> = Rc::downgrade(&rc_this) as _;
            rc_borrow.field.collision_component = Some(weak_this);
            let rc_field = Rc::new(RefCell::new(rc_borrow.field.clone()));
            edges_insert_square(
                rc_borrow.y_min,
                rc_borrow.x_min,
                rc_borrow.y_max,
                rc_borrow.x_max,
                Option::None,
                Some(rc_field),
                &mut state.pb_game_state,
            )
            .context("Failed to insert square")?;
        }

        Ok(rc_this)
    }
}

fn edges_insert_square(
    y0: f32,
    x0: f32,
    y1: f32,
    x1: f32,
    edge_segment: Option<Rc<RefCell<dyn IEdgeSegment>>>,
    field: Option<Rc<RefCell<FieldEffectType>>>,
    state: &mut PbGameState,
) -> Result<()> {
    if let Some(edge_manager) = state.edge_manager.as_mut() {
        let width_m = ((edge_manager.advance_x * 0.001) as i32) as f32;
        let height_m = ((edge_manager.advance_y * 0.001) as i32) as f32;
        let x_min = x0 - width_m;
        let x_max = x1 + width_m;
        let y_min = y0 - height_m;
        let y_max = y1 + height_m;

        let x_min_box = edge_manager.box_x(x_min);
        let y_min_box = edge_manager.box_y(y_min);
        let x_max_box = edge_manager.box_x(x_max);
        let y_max_box = edge_manager.box_y(y_max);

        let mut box_x = (x_min_box as f32) * edge_manager.advance_x + edge_manager.min_x;

        for index_x in x_min_box..=x_max_box {
            let mut box_y = y_min_box as f32 * edge_manager.advance_y + edge_manager.min_y;
            for index_y in y_min_box..=y_max_box {
                if x_max >= box_x
                    && x_min <= box_x + edge_manager.advance_x
                    && y_max >= box_y
                    && y_min <= box_y + edge_manager.advance_y
                {
                    if let Some(edge) = edge_segment.as_ref() {
                        edge_manager
                            .add_edge_to_box(index_x, index_y, Some(edge.clone()))
                            .context("Failed to add edge to box")?;
                    }
                    if let Some(f) = field.as_ref() {
                        edge_manager
                            .add_field_to_box(index_x, index_y, f.clone())
                            .context("Failed to add field to box")?;
                    }
                }
                box_y += edge_manager.advance_y;
            }
            box_x += edge_manager.advance_x;
        }
    }

    Ok(())
}

impl ICollisionComponent for TTableLayer {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
        distance: f32,
        edge: &TEdgeSegment,
        time_ticks: usize,
    ) {
        todo!()
    }

    fn edge_list(&mut self) -> &mut Vec<Rc<RefCell<dyn IEdgeSegment>>> {
        todo!()
    }

    fn set_AABB(&mut self, aabb: RectF) {
        todo!()
    }

    fn get_AABB(&self) -> Option<RectF> {
        todo!()
    }
}

pub fn edges_insert_circle(
    circle: &CircleType,
    edge: Option<Rc<RefCell<dyn IEdgeSegment>>>,
    field: Option<Rc<RefCell<FieldEffectType>>>,
    state: &mut PbGameState,
) -> Result<()> {
    let mut ray: RayType = Default::default();
    let mut vec1: Vector2 = Default::default();

    let mut radius_m: f32 = 0.0f32;
    if let Some(edge_manager) = state.edge_manager.as_ref() {
        radius_m = f32::sqrt(circle.radius_sq) + edge_manager.advance_x * 0.001;
    }

    let radius_m_sq = radius_m * radius_m;

    let x_min = circle.center.x - radius_m;
    let y_min = circle.center.y - radius_m;
    let x_max = radius_m + circle.center.x;
    let y_max = radius_m + circle.center.y;

    let mut x_min_box: i32 = 0;
    let mut y_min_box: i32 = 0;
    let mut x_max_box: i32 = 0;
    let mut y_max_box: i32 = 0;

    if let Some(edge_manager) = state.edge_manager.as_ref() {
        x_min_box = edge_manager.box_x(x_min);
        y_min_box = edge_manager.box_y(y_min);
        x_max_box = edge_manager.box_x(x_max);
        y_max_box = edge_manager.box_y(y_max);
    }

    let dir_x = if x_min_box - 1 <= 0 { 0 } else { x_min_box - 1 };
    let dir_y = if y_min_box - 1 <= 0 { 0 } else { y_min_box - 1 };

    if let Some(edge_manager) = state.edge_manager.as_ref() {
        x_max_box = edge_manager.increment_box_x(x_max_box);
        y_max_box = edge_manager.increment_box_y(y_max_box);
    }

    if let Some(edge_manager) = state.edge_manager.as_mut() {
        vec1.x = dir_x as f32 * edge_manager.advance_x + edge_manager.min_x;
        for index_x in dir_x..=x_max_box {
            vec1.y = dir_y as f32 * edge_manager.advance_y + edge_manager.min_y;
            for index_y in dir_y..=y_max_box {
                let vec1_x_adv = vec1.x + edge_manager.advance_x;
                let vec1_y_adv = vec1.y + edge_manager.advance_y;

                if x_max >= vec1.x && x_min <= vec1_x_adv && y_max >= vec1.y && y_min <= vec1_y_adv
                {
                    let mut collision = true;
                    {
                        if circle.center.x <= vec1_x_adv
                            && circle.center.x >= vec1.x
                            && circle.center.y <= vec1_y_adv
                            && circle.center.y >= vec1.y
                        {
                            break;
                        }

                        let mut vec2 = vec1;
                        if maths::distance_squared(&vec1, &circle.center) <= radius_m_sq {
                            break;
                        }

                        vec2.x += edge_manager.advance_x;
                        if maths::distance_squared(&vec2, &circle.center) <= radius_m_sq {
                            break;
                        }

                        vec2.y += edge_manager.advance_y;
                        if maths::distance_squared(&vec2, &circle.center) <= radius_m_sq {
                            break;
                        }

                        vec2.x -= edge_manager.advance_x;
                        if maths::distance_squared(&vec2, &circle.center) <= radius_m_sq {
                            break;
                        }

                        ray.origin = vec1;
                        ray.direction.x = 1.0;
                        ray.direction.y = 0.0;
                        ray.max_distance = edge_manager.advance_x;
                        if maths::ray_intersect_circle(&ray, circle) < 1000000000.0f32 {
                            break;
                        }

                        ray.direction.x = -1.0f32;
                        ray.origin.x = ray.origin.x + edge_manager.advance_x;
                        if maths::ray_intersect_circle(&ray, circle) < 1000000000.0f32 {
                            break;
                        }

                        ray.direction.x = 0.0;
                        ray.direction.y = 1.0;
                        ray.max_distance = edge_manager.advance_y;
                        if maths::ray_intersect_circle(&ray, circle) < 1000000000.0f32 {
                            break;
                        }

                        ray.direction.y = -1.0;
                        ray.origin.y += edge_manager.advance_y;
                        if maths::ray_intersect_circle(&ray, circle) < 1000000000.0f32 {
                            break;
                        }

                        ray.direction.y = 0.0;
                        ray.direction.x = -1.0;
                        ray.max_distance = edge_manager.advance_x;
                        if maths::ray_intersect_circle(&ray, circle) < 1000000000.0f32 {
                            break;
                        };

                        ray.direction.x = 1.0;
                        ray.origin.x = ray.origin.x - edge_manager.advance_x;
                        if maths::ray_intersect_circle(&ray, circle) < 1000000000.0f32 {
                            break;
                        }

                        ray.direction.x = 0.0;
                        ray.direction.y = -1.0;
                        ray.max_distance = edge_manager.advance_y;
                        if maths::ray_intersect_circle(&ray, circle) < 1000000000.0f32 {
                            break;
                        }

                        ray.direction.y = 1.0;
                        ray.origin.y -= edge_manager.advance_y;
                        if maths::ray_intersect_circle(&ray, circle) < 1000000000.0f32 {
                            break;
                        }

                        collision = false;
                    }
                    if collision {
                        if let Some(e) = edge.as_ref() {
                            edge_manager
                                .add_edge_to_box(index_x, index_y, Some(e.clone()))
                                .context("Failed to add edge to box")?;
                        }

                        if let Some(f) = field.as_ref() {
                            edge_manager
                                .add_field_to_box(index_x, index_y, f.clone())
                                .context("Failed to add field to box")?;
                        }
                    }
                }
                vec1.y += edge_manager.advance_y;
            }
            vec1.x += edge_manager.advance_x;
        }
    }

    Ok(())
}
