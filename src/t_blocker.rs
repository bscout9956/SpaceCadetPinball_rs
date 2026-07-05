use crate::loader::VisualStruct;
use crate::maths::{RectF, Vector2};
use crate::message_code::MessageCode;
use crate::render::RenderSprite;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_manager::TEdgeManager;
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_pinball_component::IPinballComponent;
use crate::t_pinball_table::TPinballTable;
use crate::utils::DrawContext;
use crate::{control, loader, timer};
use anyhow::Result;
use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::ops::Deref;
use std::rc::{Rc, Weak};

pub struct TBlocker {
    pub base: TCollisionComponent,
    pub initial_duration: i32,
    pub extended_duration: i32,
    pub timer: i32,
    pub sound_index_4: i32,
    pub sound_index_3: i32,
}

impl TBlocker {
    fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<Self> {
        let base = TCollisionComponent::new(table, group_index, true, state)?;
        base.borrow_mut().threshold = 1000000000.0f32;
        base.borrow_mut().message_field = MessageCode(0);
        base.borrow_mut().sprite_set(-1);

        let mut visual = VisualStruct::default();
        loader::query_visual(group_index, 0, &mut visual, state)?;

        let base_strong = Rc::deref(&base).clone().into_inner();

        Ok(Self {
            base: base_strong,
            initial_duration: 55,
            extended_duration: 5,
            timer: 0,
            sound_index_4: visual.sound_index_4,
            sound_index_3: visual.sound_index_3,
        })
    }

    pub unsafe extern "C" fn timer_expired(
        timer_id: i32,
        caller: *mut c_void,
        _ctx: &mut DrawContext,
    ) -> Result<()> {
        println!("TBlocker timer");
        unsafe {
            let blocker = &mut *(caller as *mut TBlocker);
            (*blocker).timer = 0;
            control::handler(MessageCode::CONTROL_TIMER_EXPIRED, blocker);
        }
        Ok(())
    }
}

impl IPinballComponent for TBlocker {
    fn render_sprite(&self) -> Option<&RenderSprite> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn group_name(&self) -> Option<Rc<RefCell<String>>> {
        self.base.group_name.clone()
    }

    fn group_index(&self) -> i32 {
        todo!()
    }

    fn sprite_set(&mut self, index: i32) {
        todo!()
    }

    fn get_coordinates(&self, tedge_manager: &TEdgeManager) -> Vector2 {
        self.base.get_coordinates(tedge_manager)
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
        draw_context: &mut DrawContext,
    ) -> Result<i32> {
        match code {
            MessageCode::SET_TILT_LOCK
            | MessageCode::PLAYER_CHANGED
            | MessageCode::RESET
            | MessageCode::T_BLOCKER_DISABLE => {
                if self.timer > 0 {
                    draw_context.timer_manager.kill_id(self.timer)?;
                    self.timer = 0;
                }
                self.base.message_field = MessageCode(0);
                self.base.active_flag.set(false);
                self.base.sprite_set(-1);
                if code == MessageCode::T_BLOCKER_DISABLE {
                    // TODO: loader::play_sound(self.sound_index_3, self, "TBlocker1");
                }
            }
            MessageCode::T_BLOCKER_ENABLE => {
                self.base.active_flag.set(true);
                // TODO: loader::play_sound(self.sound_index_4, self, "TBlocker2");
                self.base.sprite_set(0);
                if self.timer > 0 {
                    draw_context.timer_manager.kill_id(self.timer)?;
                }
                self.timer = 0;
                if value >= 0.0f32 {
                    unsafe {
                        let tm_ptr: *mut _ = &mut draw_context.timer_manager;
                        self.timer = (*tm_ptr).set(
                            value,
                            self as *mut _ as *mut c_void,
                            Self::timer_expired,
                            draw_context,
                        )?;
                    }
                }
            }
            MessageCode::T_BLOCKER_RESTART_TIMEOUT => {
                if self.timer > 0 {
                    draw_context.timer_manager.kill_id(self.timer)?;
                }
                unsafe {
                    let tm_ptr: *mut _ = &mut draw_context.timer_manager;
                    self.timer = (*tm_ptr).set(
                        f32::max(value, 0.0f32),
                        self as *mut _ as *mut c_void,
                        Self::timer_expired,
                        draw_context,
                    )?;
                }
            }
            _ => {}
        }

        Ok(0)
    }

    fn set_active_flag(&mut self, active: bool) {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl ICollisionComponent for TBlocker {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
        distance: f32,
        edge: &TEdgeSegment,
        time_ticks: &mut DrawContext,
    ) -> Result<()> {
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
