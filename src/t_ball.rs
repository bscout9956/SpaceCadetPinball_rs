use crate::loader::VisualStruct;
use crate::maths::*;
use crate::render::{RenderSprite, VisualTypes};
use crate::state::pinball_state::PinballState;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use crate::{loader, proj};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::ffi::CString;
use std::ptr::slice_from_raw_parts;
use std::rc::{Rc, Weak};
use std::sync::Arc;

#[derive(Clone)]
pub struct TBall {
    pub base_component: TPinballComponent,
    pub base_segment: TEdgeSegment,
    pub position: Vector3,
    pub prev_position: Vector3,
    pub direction: Vector3,
    pub speed: f32,
    pub ray_max_distance: f32,
    pub time_delta: f32,
    pub ramp_field_force: Vector2,
    pub collision_comp: Option<Weak<RefCell<TCollisionComponent>>>,
    pub collision_mask: i32,
    pub collisions: [Option<Weak<RefCell<dyn IEdgeSegment>>>; 16],
    pub edge_collision_count: i32,
    pub edge_collision_reset_flag: bool,
    pub collision_offset: Vector3,
    pub collision_flag: i32,
    pub radius: f32,
    pub has_group_flag: bool,
    pub stuck_count: i32,
    pub last_active_time: usize,
    pub visual_z_array: [f32; 50],
    pub collision_disabled_flag: bool,
}

impl TBall {
    pub fn new(
        table_weak: Option<Weak<RefCell<TPinballTable>>>,
        mut group_index: i32,
        state: &mut PinballState,
    ) -> Result<Rc<RefCell<Self>>> {
        let base_component = TPinballComponent::new(table_weak, group_index, false, state)?;

        let base_segment = TEdgeSegment::new(
            None,
            Rc::new(Cell::new(base_component.active_flag.get())),
            0,
        );

        let mut visual = VisualStruct::default();
        let ball_group_name = "ball";

        let mut instance_data = Self {
            base_component,
            base_segment,
            position: Default::default(),
            prev_position: Default::default(),
            direction: Default::default(),
            speed: 0.0,
            ray_max_distance: 0.0,
            time_delta: 0.0,
            ramp_field_force: Vector2 { x: 0.0, y: 0.0 },
            collision_comp: None,
            collision_mask: 0,
            collisions: [const { None }; 16],
            edge_collision_count: 0,
            edge_collision_reset_flag: false,
            collision_offset: Default::default(),
            collision_flag: 0,
            radius: 0.0,
            has_group_flag: false,
            stuck_count: 0,
            last_active_time: 0,
            visual_z_array: [0.0; 50],
            collision_disabled_flag: false,
        };

        if group_index == -1 {
            instance_data.has_group_flag = false;
            instance_data.collision_mask = 1;
        } else {
            instance_data.has_group_flag = true;
            loader::query_visual(group_index, 0, &mut visual, state)
                .context("Failed querying visual in TBall")?;
            instance_data.collision_mask = visual.collision_group;
            let float_arr_ptr =
                loader::query_float_attribute_ptr(group_index, 0, 408, &mut state.loader_state)?;
            let float_slice = slice_from_raw_parts(float_arr_ptr, 4);
            unsafe {
                let (position_x, position_y, position_z) =
                    (&(*float_slice)[0], &(*float_slice)[1], &(*float_slice)[3]);
                instance_data.position = Vector3 {
                    x: *position_x,
                    y: *position_y,
                    z: *position_z,
                }
            }
        }

        let name_bytes = ball_group_name.as_bytes().to_vec();

        let c_name = CString::new(name_bytes.clone()).expect("Null byte found");
        group_index = loader::query_handle(c_name.as_ptr(), &mut state.loader_state)
            .context("Failed to query handle in TBall")?;

        if group_index < 0 {
            let new_name = format!("ball{}", state.fullscrn_state.resolution);

            let new_c_name =
                CString::new(new_name).context("Failed to create new name string in TBall")?;
            group_index = loader::query_handle(new_c_name.as_ptr(), &mut state.loader_state)
                .context("Failed to query handle in TBall")?;
        }

        let radius_val =
            loader::query_float_attribute_ptr(group_index, 0, 500, &mut state.loader_state)
                .context("Failed to query float attribute ptr in TBall")?;
        unsafe {
            instance_data.radius = *(radius_val);
        }

        let visual_count = loader::query_visual_states(group_index, &mut state.loader_state)
            .context("Failed to query visual state in TBall::new()")?;

        for index in 0..visual_count {
            loader::query_visual(group_index, index as i32, &mut visual, state)
                .context("Querying visual")?;
            instance_data
                .base_component
                .list_bitmap
                .push(visual.bitmap.clone());
            let vis_vec_ptr = loader::query_float_attribute_ptr(
                group_index,
                index as i32,
                501,
                &mut state.loader_state,
            )
            .context("Failed to query visual in TBall")?;
            let vis_vec_slice = slice_from_raw_parts(vis_vec_ptr, 3);
            unsafe {
                let vis_vec = Vector3 {
                    x: (*vis_vec_slice)[0],
                    y: (*vis_vec_slice)[1],
                    z: (*vis_vec_slice)[2],
                };
                let z_depth = proj::z_distance(&vis_vec);
                instance_data.visual_z_array[index as usize] = z_depth;
            }
        }

        instance_data.base_component.render_sprite = Some(RenderSprite::new(
            VisualTypes::Ball,
            Arc::new(None),
            Arc::new(None),
            0,
            0,
            None,
            &mut state.render_state,
        ));

        instance_data.position.z = instance_data.radius;
        instance_data.base_component.group_index = group_index;

        let rc_instance = Rc::new(RefCell::new(instance_data));
        let weak_rc_instance =
            Rc::downgrade(&rc_instance) as Weak<RefCell<dyn ICollisionComponent>>;

        rc_instance.borrow_mut().base_segment.collision_component = Some(weak_rc_instance);

        Ok(rc_instance)
    }

    pub fn disable(&mut self) {
        self.base_component.active_flag.set(false);
        self.collision_disabled_flag = false;
        self.base_component.sprite_set(-1);
    }

    pub(crate) fn repaint(&mut self) {
        if self.collision_flag > 0 {
            self.position.z = self.collision_offset.x * self.position.x
                + self.collision_offset.y * self.position.y
                + self.radius
                + self.collision_offset.z;
        }

        let pos = Vector2 {
            x: self.position.x,
            y: self.position.y,
        };
        let mut pos_2d = proj::x_form_to_2d(&pos);
        let z_depth = proj::z_distance(&self.position);

        let mut index_set = 0;
        for index in 0..self.base_component.list_bitmap.len() - 1 {
            if self.visual_z_array[index] <= z_depth {
                index_set += 1;
                break;
            }
        }

        self.base_component
            .sprite_set_ball(index_set, &mut pos_2d, z_depth);
    }

    pub(crate) fn already_hit(&self, edge: &Rc<RefCell<dyn IEdgeSegment>>) -> bool {
        for i in 0..self.edge_collision_count {
            let downgrade = Rc::downgrade(&edge);
            if let Some(col) = self.collisions[i as usize].as_ref() {
                if col.ptr_eq(&downgrade) {
                    return true;
                }
            }
        }

        false
    }
}

impl ICollisionComponent for TBall {
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

    fn field_effect(&mut self, ball: &TBall, vec_destination: &mut Vector2) -> i32 {
        todo!()
    }

    fn set_AABB(&mut self, aabb: RectF) {
        if let Some(rc) = self.collision_comp.as_ref().and_then(|weak| weak.upgrade()) {
            rc.borrow_mut().aabb = aabb;
        }
    }

    fn get_AABB(&self) -> Option<RectF> {
        let rc = self.collision_comp.as_ref()?.upgrade()?;
        Some(rc.borrow().aabb.clone())
    }
}

use crate::state::pb_game_state::PbGameState;
use anyhow::{Context, Result};

impl IEdgeSegment for TBall {
    fn active_flag(&self) -> Rc<Cell<bool>> {
        self.base_segment.active_flag()
    }

    fn edge_collision(&self, ball: &Rc<RefCell<TBall>>, distance: f32) {
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
        todo!()
    }

    fn find_collision_distance(&self, ray: &RayType) -> f32 {
        todo!()
    }

    fn collision_group(&self) -> u32 {
        self.base_segment.collision_group
    }

    fn processed_flag(&self) -> Rc<Cell<bool>> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }
}

impl IPinballComponent for TBall {
    fn render_sprite(&self) -> Option<&RenderSprite> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn group_name(&self) -> Option<Rc<RefCell<String>>> {
        self.base_component.group_name.clone()
    }

    fn group_index(&self) -> i32 {
        todo!()
    }

    fn sprite_set(&mut self, index: i32) {
        todo!()
    }

    fn get_coordinates(&self) -> Vector2 {
        todo!()
    }

    fn get_scoring(&self, index: u32) -> i32 {
        todo!()
    }

    fn port_draw(&self) {
        todo!()
    }

    fn message(&mut self, code: MessageCode, _value: f32, _time_ticks: usize) -> i32 {
        if code == MessageCode::RESET {
            self.base_component
                .sprite_set_ball(-1, &mut Vector2i::new(0, 0), 0.0f32);
            self.position.x = 0.0;
            self.collision_comp = None;
            self.position.y = 0.0;
            self.set_active_flag(false);
            self.collision_flag = 0;
            self.collision_mask = 0;
            self.direction.y = 0.0;
            self.position.z = self.radius;
            self.direction.x = 0.0;
            self.speed = 0.0;
            self.ray_max_distance = 0.0;
        }
        0
    }

    fn set_active_flag(&mut self, active: bool) {
        todo!()
    }
}
