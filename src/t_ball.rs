use crate::loader::VisualStruct;
use crate::maths::*;
use crate::render::{RenderSprite, VisualTypes};
use crate::state::pinball_state::PinballState;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_line::EdgeSegmentError;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use crate::{loader, proj};
use std::cell::{Cell, RefCell};
use std::ffi::CString;
use std::ptr::slice_from_raw_parts;
use std::rc::{Rc, Weak};

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
    pub collisions: [Option<Weak<RefCell<TEdgeSegment>>>; 16],
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
    ) -> Rc<RefCell<Self>> {
        let base_component =
            TPinballComponent::new(table_weak, group_index, false, &mut state.loader_state);

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
            loader::query_visual(group_index, 0, &mut visual, state);
            instance_data.collision_mask = visual.collision_group;
            let float_arr_ptr =
                loader::query_float_attribute_ptr(group_index, 0, 408, &mut state.loader_state)
                    .unwrap();
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
        group_index = loader::query_handle(c_name.as_ptr(), &mut state.loader_state).unwrap();

        if group_index < 0 {
            let new_name = format!("ball{}", state.fullscrn_state.resolution);

            let new_c_name = CString::new(new_name).unwrap();
            group_index =
                loader::query_handle(new_c_name.as_ptr(), &mut state.loader_state).unwrap();
        }

        let radius_val =
            loader::query_float_attribute_ptr(group_index, 0, 500, &mut state.loader_state)
                .unwrap();
        unsafe {
            instance_data.radius = *(radius_val);
        }

        let visual_count =
            loader::query_visual_states(group_index, &mut state.loader_state).unwrap();

        for index in 0..visual_count {
            loader::query_visual(group_index, index as i32, &mut visual, state);
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
            .unwrap();
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
            None,
            None,
            0,
            0,
            None,
            &mut state.render_state,
        ));

        if let Some(t_weak) = &instance_data.base_component.pinball_table {
            if let Some(t_rc) = t_weak.upgrade() {
                t_rc.borrow_mut().collision_comp_offset = instance_data.radius;
            }
        }

        instance_data.position.z = instance_data.radius;
        instance_data.base_component.group_index = group_index;

        let rc_instance = Rc::new(RefCell::new(instance_data));
        let weak_rc_instance =
            Rc::downgrade(&rc_instance) as Weak<RefCell<dyn ICollisionComponent>>;

        rc_instance.borrow_mut().base_segment.collision_component = Some(weak_rc_instance);

        rc_instance
    }

    pub fn disable(&mut self) {
        self.base_component.active_flag.set(false);
        self.collision_disabled_flag = false;
        self.base_component.sprite_set(-1);
    }

    // TODO: Is this part of a trait?
    pub(crate) fn repaint(&self) {
        todo!()
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

    fn field_effect(&mut self, ball: &TBall, vec_destination: &mut Vector2) -> i32 {
        todo!()
    }
}

impl IEdgeSegment for TBall {
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
        todo!()
    }

    fn find_collision_distance(&self, ray: &RayType) -> f32 {
        todo!()
    }
}
