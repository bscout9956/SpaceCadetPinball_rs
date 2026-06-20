use crate::errors::LoaderError;
use crate::gdrv::GdrvBitmap8;
use crate::loader::{VisualStruct, query_float_attribute_ptr};
use crate::maths::{MathsError, RectangleType, Vector2, f32_vec_to_vec3};
use crate::render::{RenderSprite, VisualTypes};
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::TCollisionComponent;
use crate::t_edge_manager::{FieldEffectType, TEdgeManager};
use crate::t_pinball_table::TPinballTable;
use crate::{loader, proj, render};
use anyhow::{bail, Context};
use std::cell::RefCell;
use std::f32::consts::FRAC_PI_2;
use std::ffi::CString;
use std::ptr::slice_from_raw_parts;
use std::rc::Weak;
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

use anyhow::Result;

impl TTableLayer {
    pub fn field_effect(ball: Option<&mut TBall>, vec_dst: Option<&mut Vector2>) {
        todo!("I am never finished omg");
    }

    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        state: &mut PinballState,
    ) -> Result<Self> {
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

        let table_angle_array =
            query_float_attribute_ptr(group_index, 0, 305, &mut state.loader_state)?;
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

        // TODO: Remove upgrades, breaks stuff
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

        let mut instance = Self {
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
            field: vec![],
        };

        let height = instance.x_max;
        let width = instance.y_max;

        let mut edge_manager = EDGE_MANAGER
            .lock()
            .map_err(|_| TTableLayerError::LockError)?;
        *edge_manager = Some(TEdgeManager::new(
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

        TTableLayer::new(table, state) // TODO: I'm unfinished, just so rustc can stfu
    }
}

pub static EDGE_MANAGER: Mutex<Option<TEdgeManager>> = Mutex::new(None);
