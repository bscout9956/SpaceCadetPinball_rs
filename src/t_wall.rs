use crate::maths::{RectF, Vector2, Vector2i};
use crate::message_code::MessageCode;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_pinball_component::IPinballComponent;
use crate::t_pinball_table::TPinballTable;
use crate::timer;
use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::{Rc, Weak};

pub struct TWall {
    base: Weak<RefCell<TCollisionComponent>>,
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
        time_ticks: usize,
    ) {
        if let Some(base_val) = self.base.upgrade() {
            let mut base_val_borrow = base_val.borrow_mut();
            if base_val_borrow.default_collision(ball, next_position, direction) {
                // TODO: What if this isn't enough? Is empty is an assumption
                if !base_val_borrow.base.list_bitmap.is_empty() {
                    base_val_borrow.sprite_set(0);
                    self.timer = timer::set(
                        0.1f32,
                        &raw mut *self as *mut c_void,
                        timer_expired,
                        time_ticks,
                    );
                }
                //TODO: control::handler(MessageCode::CONTROL_COLLISION, self);
            }
        }
    }

    fn edge_list(&mut self) -> &mut Vec<Rc<RefCell<dyn IEdgeSegment>>> {
        todo!()
    }
    #[allow(non_snake_case)]
    fn set_AABB(&mut self, aabb: RectF) {
        if let Some(rc) = self.base.upgrade() {
            rc.borrow_mut().aabb = aabb;
        }
    }

    #[allow(non_snake_case)]
    fn get_AABB(&self) -> Option<RectF> {
        let rc = self.base.upgrade()?;
        Some(rc.borrow().aabb.clone())
    }
}

use crate::render::RenderSprite;
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

        let downgraded_col = Rc::downgrade(&base_tcol);

        Ok(Self {
            base: downgraded_col,
            timer: 0,
        })
    }
}
unsafe extern "C" fn timer_expired(timer_id: i32, caller: *mut c_void, state: &mut PinballState) {
    let wall = caller as *mut TWall;
    if !wall.is_null() {
        unsafe {
            if let Some(base_comp) = (*wall).base.upgrade() {
                let mut borrow = base_comp.borrow_mut();
                borrow.sprite_set(-1);
                (*wall).timer = 0;
                borrow.message_field = MessageCode(0);
            }
        }
    }
}

impl IPinballComponent for TWall {
    fn render_sprite(&self) -> Option<&RenderSprite> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn group_name(&self) -> Option<Rc<RefCell<String>>> {
        self.base
            .upgrade()
            .map(|base| base.borrow().group_name.clone())?
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

    fn message(&mut self, code: MessageCode, value: f32, time_ticks: usize) -> i32 {
        todo!()
    }

    fn set_active_flag(&mut self, active: bool) {
        todo!()
    }
}
