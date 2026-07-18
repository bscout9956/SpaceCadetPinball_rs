use crate::context::component_context::ComponentContext;
use crate::control::ComponentControl;
use crate::loader::{VisualStruct, query_float_attribute_ptr};
use crate::maths::{RectF, Vector2, Vector3, f32_ptr_to_vec3};
use crate::message_code::MessageCode;
use crate::render::RenderSpriteRef;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_manager::TEdgeManager;
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_flipper_edge::TFlipperEdge;
use crate::t_pinball_component::IPinballComponent;
use crate::t_pinball_table::TPinballTable;
use crate::{loader, maths};
use anyhow::Result;
use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Default)]
pub struct TFlipper {
    base: TCollisionComponent,
    pub bmp_index: i32,
    pub t_flipper_edge: Option<Rc<RefCell<TFlipperEdge>>>,
}

impl TFlipper {
    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<Rc<RefCell<Self>>> {
        let mut base = TCollisionComponent::new(table.clone(), group_index, false, state)?.take();
        let mut visual = VisualStruct::default();

        loader::query_visual(group_index, 0, &mut visual, state)?;

        base.hard_hit_sound_id = visual.sound_index_4;
        base.soft_hit_sound_id = visual.sound_index_3;
        base.elasticity = visual.elasticity;
        base.smoothness = visual.smoothness;

        let col_mult =
            unsafe { *query_float_attribute_ptr(group_index, 0, 803, &mut state.loader_state)? };
        let retract_time =
            unsafe { *query_float_attribute_ptr(group_index, 0, 805, &mut state.loader_state)? };
        let extend_time =
            unsafe { *query_float_attribute_ptr(group_index, 0, 804, &mut state.loader_state)? };

        let vec_t2 = unsafe {
            f32_ptr_to_vec3(query_float_attribute_ptr(
                group_index,
                0,
                802,
                &mut state.loader_state,
            )?)?
        };
        let vec_t1 = unsafe {
            f32_ptr_to_vec3(query_float_attribute_ptr(
                group_index,
                0,
                801,
                &mut state.loader_state,
            )?)?
        };
        let origin = unsafe {
            f32_ptr_to_vec3(query_float_attribute_ptr(
                group_index,
                0,
                800,
                &mut state.loader_state,
            )?)?
        };

        let active_flag = base.active_flag.clone();
        let elasticity = base.elasticity;
        let smoothness = base.smoothness;
        let instance = Rc::new(RefCell::new(Self {
            base,
            bmp_index: 0,
            t_flipper_edge: None,
        }));

        let weak_component = Rc::downgrade(&instance) as Weak<RefCell<dyn ICollisionComponent>>;
        let flipper_edge = Rc::new(RefCell::new(TFlipperEdge::new(
            Some(weak_component),
            active_flag,
            visual.collision_group as u32,
            table.clone(),
            origin,
            vec_t1,
            vec_t2,
            extend_time,
            retract_time,
            col_mult,
            elasticity,
            smoothness,
            state,
        )));
        let edge: Rc<RefCell<dyn IEdgeSegment>> = flipper_edge.clone();

        {
            let mut flipper = instance.borrow_mut();
            let mut aabb = flipper.base.aabb.clone();
            flipper_edge.borrow().place_in_grid(
                &mut aabb,
                Some(edge.clone()),
                &mut state.pb_game_state,
            )?;
            flipper.base.aabb = aabb;
            flipper.base.edge_list.push(edge);
            flipper.t_flipper_edge = Some(flipper_edge);
        }

        if let Some(table) = table.and_then(|table| table.upgrade()) {
            table.borrow_mut().flipper_list.push(instance.clone());
        }

        Ok(instance)
    }

    pub(crate) fn flipper_collision(&self, _p0: f32) {
        todo!()
    }
}

impl TFlipper {
    pub(crate) fn update_sprite(&mut self) {
        let bmp_count_sub_1 = (self.base.list_bitmap.len() - 1) as i32;
        if let Some(flipper_edge) = self.t_flipper_edge.as_ref() {
            let mut new_bmp_index = f32::floor(
                flipper_edge.borrow().current_angle / flipper_edge.borrow().angle_max
                    * bmp_count_sub_1 as f32
                    + 0.5,
            ) as i32;
            new_bmp_index = utils::clamp(&new_bmp_index, &0, &bmp_count_sub_1);
            if self.bmp_index == new_bmp_index {
                return;
            }

            self.bmp_index = new_bmp_index;
            self.sprite_set(self.bmp_index);
        }
    }
}

// TODO: Implement me asap
impl TFlipper {
    pub(crate) fn get_flipper_step_angle(&self, dt: f32, dst: &mut f32) -> i32 {
        if self.base.message_field == MessageCode::T_FLIPPER_NULL {
            return 0;
        }

        if let Some(flipper_edge) = self.t_flipper_edge.as_ref() {
            let delta_angle = flipper_edge.borrow().flipper_angle_delta(dt);
            let mut step = f32::abs(f32::ceil(
                flipper_edge.borrow().distance_div
                    * delta_angle
                    * flipper_edge.borrow().inv_t1_radius,
            ));
            if step > 3.0 {
                step = 3.0;
            }
            if step >= 2.0 {
                *dst = delta_angle / step;
                return step as i32;
            }

            *dst = delta_angle;
        }
        return 1;
    }
}

impl IPinballComponent for TFlipper {
    fn render_sprite(&self) -> Option<RenderSpriteRef> {
        self.base.render_sprite.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn group_name(&self) -> Option<Rc<RefCell<String>>> {
        self.base.group_name.clone()
    }

    fn group_index(&self) -> i32 {
        self.base.group_index
    }

    fn sprite_set(&mut self, index: i32) {
        self.base.sprite_set(index);
    }

    fn get_coordinates(&self, edge_manager: &TEdgeManager) -> Vector2 {
        self.base.get_coordinates(edge_manager)
    }

    fn get_scoring(&self, index: u32) -> i32 {
        self.base.get_scoring(index)
    }

    fn port_draw(&self) {
        self.base.port_draw()
    }

    fn message(
        &mut self,
        code: &mut MessageCode,
        value: f32,
        component_context: &mut ComponentContext,
    ) -> Result<i32> {
        self.base.message(code, value, component_context)
    }

    fn set_active_flag(&mut self, active: bool) {
        self.base.set_active_flag(active);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_collision_component(&self) -> Option<&TCollisionComponent> {
        Some(&self.base)
    }

    fn set_control(&mut self, control: Option<Weak<RefCell<ComponentControl>>>) {
        self.base.set_control(control);
    }
}

impl ICollisionComponent for TFlipper {
    fn collision(
        &mut self,
        _ball: &mut TBall,
        _next_position: &Vector2,
        _direction: &mut Vector2,
        _distance: f32,
        _edge: &dyn IEdgeSegment,
        _component_context: &mut ComponentContext,
    ) -> Result<()> {
        Ok(())
    }

    fn edge_list(&mut self) -> &mut Vec<Rc<RefCell<dyn IEdgeSegment>>> {
        &mut self.base.edge_list
    }

    fn field_effect(
        &mut self,
        _ball_position: &Vector3,
        _ball_direction: &Vector3,
        _ball_speed: f32,
        _vec_destination: &mut Vector2,
    ) -> i32 {
        0
    }

    fn set_AABB(&mut self, aabb: RectF) {
        self.base.aabb = aabb;
    }

    fn get_AABB(&self) -> Option<RectF> {
        Some(self.base.aabb.clone())
    }
}
