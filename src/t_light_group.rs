use crate::errors::LoaderError;
use crate::control::ComponentControl;
use crate::loader;
use crate::maths::Vector2;
use crate::message_code::MessageCode;
use crate::state::pinball_state::PinballState;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use anyhow::Result;

use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use thiserror::Error;

#[derive(Default)]
struct TLightGroupPlayerBackup {
    message_field: i32,
    message_field_2: MessageCode,
    timer1time: f32,
    unknown3: i32,
}

pub struct TLightGroup {
    base: TPinballComponent,
    id_list: Vec<i32>, // Equivalent to list
    timer1time: f32,
    timer1time_default: f32,
    message_field_2: MessageCode,
    animation_flag: bool,
    notify_timer: i32,
    timer: i32,
    player_data: [TLightGroupPlayerBackup; 4],
}

impl TLightGroup {
    fn reset(&mut self, cmp_ctx: &mut ComponentContext) -> Result<()> {
        if self.timer > 0 {
            cmp_ctx.timer_manager.borrow_mut().kill_id(self.timer)?;
        }
        self.timer = 0;
        if self.notify_timer > 0 {
            cmp_ctx
                .timer_manager
                .borrow_mut()
                .kill_id(self.notify_timer)?;
        }
        self.notify_timer = 0;
        self.message_field_2 = MessageCode::T_LIGHT_GROUP_NULL;
        self.animation_flag = false;
        self.timer1time = self.timer1time_default;
        Ok(())
    }

    pub(crate) fn next_light_up(&self) -> i32 {
        let components = self.get_light_components();
        for (index, comp) in components.iter().enumerate() {
            if let Some(light) = comp.borrow().as_tlight() {
                if !light.light_on_flag {
                    return index as i32;
                }
            }
        }
        -1
    }

    pub(crate) fn next_light_down(&self) -> i32 {
        let components = self.get_light_components();
        for (index, comp) in components.iter().enumerate().rev() {
            if let Some(light) = comp.borrow().as_tlight() {
                if light.light_on_flag {
                    return index as i32;
                }
            }
        }
        -1
    }

    fn get_light_components(&self) -> Vec<Rc<RefCell<dyn IPinballComponent>>> {
        if let Some(table_rc) = self.base.pinball_table.as_ref().and_then(|t| t.upgrade()) {
            let table = table_rc.borrow();
            self.id_list
                .iter()
                .filter_map(|&id| table.find_component(id))
                .filter(|component| component.borrow().as_tlight().is_some())
                .collect()
        } else {
            vec![]
        }
    }

    fn get_light_component_by_position(
        &self,
        position: usize,
    ) -> Option<Rc<RefCell<dyn IPinballComponent>>> {
        self.get_light_components().get(position).cloned()
    }

    fn get_self_component(&self) -> Option<Rc<RefCell<dyn IPinballComponent>>> {
        let table_rc = self.base.pinball_table.as_ref().and_then(|t| t.upgrade())?;
        let table = table_rc.borrow();
        table
            .component_list
            .iter()
            .find(|component| component.borrow().group_index() == self.base.group_index)
            .cloned()
    }

    fn dispatch_control(&self, code: MessageCode, ctx: &mut ComponentContext) -> Result<()> {
        let Some(control) = self.base.control.as_ref().and_then(|control| control.upgrade()) else {
            return Ok(());
        };
        let Some(caller) = self.get_self_component() else {
            return Ok(());
        };

        let control_func = control.borrow().control_func;
        control_func(code, caller, ctx.main_table.clone(), ctx.full_tilt_mode)
    }

    fn reschedule_animation(&mut self, time: f32, ctx: &mut ComponentContext) -> Result<()> {
        if self.timer > 0 {
            ctx.timer_manager.borrow_mut().kill_id(self.timer)?;
        }
        self.timer = 0;

        if time == 0.0 {
            self.message_field_2 = MessageCode::T_LIGHT_GROUP_NULL;
            self.animation_flag = false;
            return Ok(());
        }

        self.timer1time = if time > 0.0 {
            time
        } else {
            self.timer1time_default
        };
        self.timer = ctx.set_timer(
            self.timer1time,
            &raw mut *self as *mut c_void,
            timer_expired,
        )?;
        Ok(())
    }

    pub fn start_animation(&mut self, ctx: &mut ComponentContext) -> Result<()> {
        let lights = self.get_light_components();
        for light in lights.iter().rev() {
            let mut light_borrow = light.borrow_mut();
            if let Some(l) = light_borrow.as_tlight_mut() {
                if l.light_on_flag {
                    l.message(MessageCode::T_LIGHT_TURN_ON_TIMED, 0.0, ctx)?;
                } else {
                    l.message(MessageCode::T_LIGHT_TURN_OFF_TIMED, 0.0, ctx)?;
                }
            }
        }
        Ok(())
    }
}

impl IPinballComponent for TLightGroup {
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
        _code: MessageCode,
        value: f32,
        _ctx: &mut ComponentContext,
    ) -> Result<i32> {
        println!("BLINKY BLINKY OOO");
        Ok(0)
    }

    fn set_active_flag(&mut self, active: bool) {
        self.base.set_active_flag(active);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn set_control(&mut self, control: Option<Weak<RefCell<ComponentControl>>>) {
        self.base.set_control(control);
    }
}

unsafe extern "C" fn timer_expired(
    _timer_id: i32,
    caller: *mut c_void,
    ctx: &mut ComponentContext,
) -> Result<()> {
    unsafe {
        let group = caller as *mut TLightGroup;
        (*group).timer = 0;
        (*group).message((*group).message_field_2, (*group).timer1time, ctx)?;
    }
    Ok(())
}

unsafe extern "C" fn notify_timer_expired(
    _timer_id: i32,
    caller: *mut c_void,
    ctx: &mut ComponentContext,
) -> Result<()> {
    unsafe {
        let group = caller as *mut TLightGroup;
        (*group).notify_timer = 0;
        (*group).dispatch_control(MessageCode::CONTROL_NOTIFY_TIMER_EXPIRED, ctx)?;
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum TLightGroupError {
    #[error("Error loading data `{0}`")]
    LoaderError(#[from] LoaderError),
}

use crate::context::component_context::ComponentContext;
use crate::render::RenderSpriteRef;
use crate::t_edge_manager::TEdgeManager;

impl TLightGroup {
    pub(crate) fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<TLightGroup> {
        let base = TPinballComponent::new(table, group_index, false, state)?;

        let mut instance = Self {
            base,
            id_list: vec![],
            timer1time: 0.0,
            timer1time_default: 0.0,
            message_field: 0,
            message_field_2: Default::default(),
            animation_flag: false,
            notify_timer: 0,
            timer: 0,
            player_data: std::array::from_fn(|_| TLightGroupPlayerBackup::default()),
        };

        if group_index > 0 {
            let mut count = 0;
            instance.timer1time_default = unsafe {
                *loader::query_float_attribute_ptr(group_index, 0, 903, &mut state.loader_state)?
            };
            let mut group_idx_ptr = loader::query_int_attribute(
                group_index,
                1027,
                &raw mut count,
                &mut state.loader_state,
            )?;
            for _ in 0..count {
                let light_idx = unsafe { (*group_idx_ptr) as i32 };
                instance.id_list.push(light_idx);
                unsafe { group_idx_ptr = group_idx_ptr.add(1) }
            }
        }

        Ok(instance)
    }
}
