use crate::maths::{RectF, Vector2};
use crate::message_code::MessageCode;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_pinball_component::IPinballComponent;
use crate::t_pinball_table::TPinballTable;
use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::{Rc, Weak};

pub struct TWall {
    base: TCollisionComponent,
    pub timer: i32,
}

impl ICollisionComponent for TWall {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
        distance: f32,
        edge: &TEdgeSegment,
        draw_context: &mut ComponentContext,
    ) -> Result<()> {
        if !self.base.list_bitmap.is_empty() {
            self.base.sprite_set(0);
            self.timer = draw_context.timer_manager.borrow_mut().set(
                0.1f32,
                &raw mut *self as *mut c_void,
                timer_expired,
                draw_context,
            )?;
        }
        Ok(())
        //TODO: control::handler(MessageCode::CONTROL_COLLISION, self);
    }

    fn edge_list(&mut self) -> &mut Vec<Rc<RefCell<dyn IEdgeSegment>>> {
        todo!()
    }
    #[allow(non_snake_case)]
    fn set_AABB(&mut self, aabb: RectF) {
        self.base.aabb = aabb;
    }

    #[allow(non_snake_case)]
    fn get_AABB(&self) -> Option<RectF> {
        Some(self.base.aabb.clone())
    }
}

use crate::context::component_context::ComponentContext;
use crate::render::RenderSprite;
use crate::t_edge_manager::TEdgeManager;
use crate::timer::TimerManager;
use anyhow::Result;

impl TWall {
    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<Self> {
        let base_tcol = TCollisionComponent::new(table, group_index, true, state)?;
        if base_tcol.borrow_mut().base.render_sprite.is_some() {
            base_tcol.borrow_mut().base.sprite_set(-1);
        }

        let base = Rc::unwrap_or_clone(base_tcol).into_inner();

        Ok(Self { base, timer: 0 })
    }
}

impl IPinballComponent for TWall {
    fn render_sprite(&self) -> Option<&RenderSprite> {
        self.base.render_sprite.as_ref()
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
        todo!()
    }

    fn get_coordinates(&self, edge_manager: &TEdgeManager) -> Vector2 {
        self.base.get_coordinates(edge_manager)
    }

    fn get_scoring(&self, index: u32) -> i32 {
        todo!()
    }

    fn port_draw(&self) {
        todo!()
    }

    fn message(
        &mut self,
        code: MessageCode,
        value: f32,
        draw_context: &mut ComponentContext,
    ) -> Result<i32> {
        if code == MessageCode::RESET && self.timer > 0 {
            draw_context
                .timer_manager
                .borrow_mut()
                .kill_id(self.timer)?;
            unsafe {
                timer_expired(self.timer, &raw mut *self as *mut c_void, draw_context)?;
            }
        }
        Ok(0)
    }

    fn set_active_flag(&mut self, active: bool) {
        self.base.active_flag.set(active);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_collision_component(&self) -> Option<&TCollisionComponent> {
        Some(&self.base)
    }
}

unsafe extern "C" fn timer_expired(
    timer_id: i32,
    caller: *mut c_void,
    _ctx: &mut ComponentContext,
) -> Result<()> {
    let wall = caller as *mut TWall;
    if !wall.is_null() {
        unsafe {
            (*wall).base.sprite_set(-1);
            (*wall).timer = 0;
        }
    }
    Ok(())
}
