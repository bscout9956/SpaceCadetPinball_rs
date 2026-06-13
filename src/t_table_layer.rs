use crate::errors::LoaderError;
use crate::gdrv::GdrvBitmap8;
use crate::loader::{VisualStruct, query_float_attribute_ptr};
use crate::maths::{MathsError, RectangleType, Vector2, f32_vec_to_vec3};
use crate::render::{RenderSprite, VisualTypes};
use crate::t_ball::TBall;
use crate::t_collision_component::TCollisionComponent;
use crate::t_edge_manager::{FieldEffectType, TEdgeManager};
use crate::t_edge_segment::IEdgeSegment;
use crate::t_line::TLine;
use crate::t_pinball_table::TPinballTable;
use crate::{fullscrn, loader, pb, proj, render};
use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;
use std::ops::Deref;
use std::ptr::slice_from_raw_parts;
use std::rc::{Rc, Weak};
use std::slice::from_raw_parts;
use std::sync::Mutex;
use std::sync::atomic::Ordering::SeqCst;
use thiserror::Error;

pub struct TTableLayer {
    pub base_component: TCollisionComponent,
    pub vis_bmp: Option<GdrvBitmap8>,
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
    pub gravity_dir_x: f32,
    pub gravity_dir_y: f32,
    pub gravity_mult: f32,
    pub field: Vec<FieldEffectType>,
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

impl TTableLayer {
    pub fn field_effect(ball: Option<&mut TBall>, vec_dst: Option<&mut Vector2>) {
        todo!("I am never finished omg");
    }

    pub fn new(table: Option<Weak<RefCell<TPinballTable>>>) -> Result<Self, TTableLayerError> {
        let mut visual = VisualStruct::default();
        let mut rect = RectangleType::default();

        let group_index = loader::query_handle(c"table".as_ptr())?;
        loader::query_visual(group_index, 0, &mut visual)?;
        let sprite_data = visual.bitmap;

        /*Full tilt: proj center first value is offset by resolution*/
        let float_ptr =
            loader::query_float_attribute_ptr(group_index, 0, 700 + fullscrn::get_resolution())?;
        let proj_center = slice_from_raw_parts(float_ptr, 2);
        unsafe {
            proj::recenter(&(*proj_center)[0], &(*proj_center)[1]);
        }
        render::set_background_zmap(sprite_data.zmap, 0, 0);

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
        );

        let table_angle_array = query_float_attribute_ptr(group_index, 0, 305)?;
        let table_slice = unsafe { from_raw_parts(table_angle_array, 3) };
        if !table_angle_array.is_null() {
            if let Some(t) = table.as_ref().unwrap().upgrade() {
                t.borrow_mut().gravity_dir_vect_mult = table_slice[0];
                t.borrow_mut().gravity_dir_vect_mult = table_slice[1];
                t.borrow_mut().gravity_dir_vect_mult = table_slice[2];
            }
        } else {
            if let Some(t) = table.as_ref().unwrap().upgrade() {
                t.borrow_mut().gravity_dir_vect_mult = 25.0f32;
                t.borrow_mut().gravity_dir_vect_mult = 0.5f32;
                t.borrow_mut().gravity_dir_vect_mult = FRAC_PI_2; // 1.570796f in the original
            }
        }

        let gravity_dir_x;
        let gravity_dir_y;

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
            return Err(TTableLayerError::InvalidTable);
        }

        let gravity_mult: f32;
        if pb::FULL_TILT_MODE.load(SeqCst) == false && pb::FULL_TILT_DEMO_MODE.load(SeqCst) == false
        {
            let angle_mult = loader::query_float_attribute_ptr(group_index, 0, 701)?;
            gravity_mult = unsafe { *angle_mult };
        } else {
            gravity_mult = 0.2f32;
        }

        let mut base = TCollisionComponent::new(table.clone(), -1, false);
        base.borrow_mut().threshold = visual.kicker.threshold;
        base.borrow_mut().boost = 15.0f32;

        let edge_points = f32_vec_to_vec3(&visual.float_arr)?;

        let mut instance = Self {
            base_component: *base.borrow(),
            vis_bmp: *bmp,
            x_min: f32::min(
                edge_points[0].x,
                f32::min(edge_points[1].x, edge_points[2].y),
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
            field: vec![],
        };

        let height = instance.x_max;
        let width = instance.y_max;

        let mut edge_manager = EDGE_MANAGER
            .lock()
            .map_err(|_| TTableLayerError::LockError)?;
        (*edge_manager) = Some(TEdgeManager::new(
            instance.x_min,
            instance.y_min,
            width,
            height,
        ));

        for i in 0..visual.float_arr_count {
            // TODO: Finish me pls
            // let line = TLine::new(/*this*/, visual.collision_group, edge_points[i + 1].x, edge_points[i + 1].y, edge_points[i].x, edge_points[i].y);
            // line.place_in_grid(&instance.base_component.AABB);
        }

        Ok(TTableLayer::new(table)?) // TODO: I'm unfinished, just so rustc can stfu
    }
}

pub static EDGE_MANAGER: Mutex<Option<TEdgeManager>> = Mutex::new(None);
