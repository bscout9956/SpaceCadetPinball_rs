use crate::control::ComponentControl;
use crate::errors::LoaderError;
use crate::maths::Vector2;
use crate::message_code::MessageCode;
use crate::state::pinball_state::PinballState;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use crate::{loader, utils};
use anyhow::Result;

use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
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
    pub list: Vec<Rc<RefCell<dyn IPinballComponent>>>,
    timer1time: f32,
    timer1time_default: f32,
    message_field: i32,
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
        self.list.clone()
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
        let Some(control) = self
            .base
            .control
            .as_ref()
            .and_then(|control| control.upgrade())
        else {
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
        code: MessageCode,
        value: f32,
        ctx: &mut ComponentContext,
    ) -> Result<i32> {
        match code {
            MessageCode::SET_TILT_LOCK | MessageCode::GAME_OVER => {}
            MessageCode::PLAYER_CHANGED => {
                if let Some(table) = ctx.main_table.as_ref() {
                    let mut player = self
                        .player_data
                        .get_mut(table.borrow_mut().current_player as usize);
                    if let Some(player) = player.as_mut() {
                        player.message_field = self.message_field;
                        player.message_field_2 = self.message_field_2;
                        player.timer1time = self.timer1time;
                    }

                    self.reset(ctx)?;

                    player = self.player_data.get_mut(value.floor() as usize);
                    if let Some(player) = player.as_mut() {
                        self.message_field = player.message_field;
                        self.message_field_2 = player.message_field_2;
                        self.timer1time = player.timer1time;
                        if self.message_field != 0 {
                            unsafe {
                                timer_expired(0, &raw mut *self as *mut c_void, ctx)?;
                            }
                        }
                    }
                }
            }
            MessageCode::RESET => {
                self.reset(ctx)?;
                if let Some(table) = ctx.main_table.as_ref() {
                    for index in 0..table.borrow().player_count {
                        let player_opt = self.player_data.get_mut(index as usize);
                        if let Some(player) = player_opt {
                            player.message_field = self.message_field;
                            player.message_field_2 = self.message_field_2;
                            player.timer1time = self.timer1time;
                        }
                    }
                }
            }
            MessageCode::T_LIGHT_GROUP_STEP_BACKWARD => {
                let lights = self.get_light_components();
                let count = lights.len();
                if count == 0 {
                    return Ok(0);
                }

                let (last_message, last_status) = {
                    let last_component = lights[count - 1].borrow();
                    let Some(last_light) = last_component.as_tlight() else {
                        return Ok(0);
                    };

                    if last_light.flasher_on_flag
                        || last_light.toggled_on_flag
                        || last_light.toggled_off_flag
                    {
                        return Ok(0);
                    }

                    (last_light.message_field(), last_light.light_on_flag)
                };

                if self.message_field_2 != MessageCode::T_LIGHT_GROUP_NULL {
                    self.message(MessageCode::T_LIGHT_GROUP_RESET, 0.0, ctx)?;
                }
                self.animation_flag = true;
                self.message_field_2 = code;

                for index in (1..count).rev() {
                    let (prev_status, prev_message) = {
                        let prev_component = lights[index - 1].borrow();
                        let Some(prev_light) = prev_component.as_tlight() else {
                            continue;
                        };
                        (prev_light.light_on_flag, prev_light.message_field())
                    };

                    let mut cur_component = lights[index].borrow_mut();
                    let Some(cur_light) = cur_component.as_tlight_mut() else {
                        continue;
                    };
                    cur_light.message(
                        if prev_status {
                            MessageCode::T_LIGHT_TURN_ON
                        } else {
                            MessageCode::T_LIGHT_TURN_OFF
                        },
                        0.0,
                        ctx,
                    )?;
                    cur_light.set_message_field(prev_message);
                }

                {
                    let mut first_component = lights[0].borrow_mut();
                    if let Some(first_light) = first_component.as_tlight_mut() {
                        first_light.message(
                            if last_status {
                                MessageCode::T_LIGHT_TURN_ON
                            } else {
                                MessageCode::T_LIGHT_TURN_OFF
                            },
                            0.0,
                            ctx,
                        )?;
                        first_light.set_message_field(last_message);
                    }
                }

                self.reschedule_animation(value, ctx)?;
            }
            MessageCode::T_LIGHT_GROUP_STEP_FORWARD => {
                let lights = self.get_light_components();
                let count = lights.len();
                if count == 0 {
                    return Ok(0);
                }

                let (first_message, first_status) = {
                    let first_component = lights[0].borrow();
                    let Some(first_light) = first_component.as_tlight() else {
                        return Ok(0);
                    };

                    if first_light.flasher_on_flag
                        || first_light.toggled_on_flag
                        || first_light.toggled_off_flag
                    {
                        return Ok(0);
                    }

                    (first_light.message_field(), first_light.light_on_flag)
                };

                if self.message_field_2 != MessageCode::T_LIGHT_GROUP_NULL {
                    self.message(MessageCode::T_LIGHT_GROUP_RESET, 0.0, ctx)?;
                }
                self.animation_flag = true;
                self.message_field_2 = code;

                for index in 0..count - 1 {
                    let (next_status, next_message) = {
                        let next_component = lights[index + 1].borrow();
                        let Some(next_light) = next_component.as_tlight() else {
                            continue;
                        };
                        (next_light.light_on_flag, next_light.message_field())
                    };

                    let mut cur_component = lights[index].borrow_mut();
                    let Some(cur_light) = cur_component.as_tlight_mut() else {
                        continue;
                    };
                    cur_light.message(
                        if next_status {
                            MessageCode::T_LIGHT_TURN_ON
                        } else {
                            MessageCode::T_LIGHT_TURN_OFF
                        },
                        0.0,
                        ctx,
                    )?;
                    cur_light.set_message_field(next_message);
                }

                {
                    let mut last_component = lights[count - 1].borrow_mut();
                    if let Some(last_light) = last_component.as_tlight_mut() {
                        last_light.message(
                            if first_status {
                                MessageCode::T_LIGHT_TURN_ON
                            } else {
                                MessageCode::T_LIGHT_TURN_OFF
                            },
                            0.0,
                            ctx,
                        )?;
                        last_light.set_message_field(first_message);
                    }
                }

                self.reschedule_animation(value, ctx)?;
            }
            MessageCode::T_LIGHT_GROUP_ANIMATION_BACKWARD => {
                if self.animation_flag || self.message_field_2 == MessageCode::T_LIGHT_GROUP_NULL {
                    self.start_animation(ctx)?;
                }
                self.message_field_2 = code;
                self.animation_flag = false;
                let components = self.get_light_components();
                if components.is_empty() {
                    return Ok(0);
                }

                let last_status = {
                    let Some(light_component) = components.last() else {
                        return Ok(0);
                    };

                    let light_borrow = light_component.borrow();

                    let Some(last_light) = light_borrow.as_tlight() else {
                        return Ok(0);
                    };

                    last_light.toggled_on_flag
                };

                for index in (1..components.len()).rev() {
                    let prev_status = {
                        let light_prev = components[index - 1].borrow();
                        let Some(light_prev) = light_prev.as_tlight() else {
                            continue;
                        };
                        light_prev.toggled_on_flag
                    };

                    let mut light_curr = components[index].borrow_mut();
                    let Some(light_curr) = light_curr.as_tlight_mut() else {
                        continue;
                    };
                    light_curr.message(
                        if prev_status {
                            MessageCode::T_LIGHT_TURN_ON_TIMED
                        } else {
                            MessageCode::T_LIGHT_TURN_OFF_TIMED
                        },
                        0.0,
                        ctx,
                    )?;
                }
                {
                    let mut first_light = components[0].borrow_mut();
                    if let Some(first_light) = first_light.as_tlight_mut() {
                        first_light.message(
                            if last_status {
                                MessageCode::T_LIGHT_TURN_ON_TIMED
                            } else {
                                MessageCode::T_LIGHT_TURN_OFF_TIMED
                            },
                            0.0,
                            ctx,
                        )?;
                    }
                }
                self.reschedule_animation(value, ctx)?;
            }
            MessageCode::T_LIGHT_GROUP_ANIMATION_FORWARD => {
                if self.animation_flag || self.message_field_2 == MessageCode::T_LIGHT_GROUP_NULL {
                    self.start_animation(ctx)?;
                }
                self.message_field_2 = code;
                self.animation_flag = false;
                let components = self.get_light_components();
                if components.is_empty() {
                    return Ok(0);
                }

                let first_status = {
                    let Some(light_component) = components.first() else {
                        return Ok(0);
                    };

                    let light_borrow = light_component.borrow();

                    let Some(first_light) = light_borrow.as_tlight() else {
                        return Ok(0);
                    };

                    first_light.toggled_on_flag
                };

                for index in 0..components.len() - 1 {
                    let next_status = {
                        let light_next = components[index + 1].borrow();
                        let Some(light_next) = light_next.as_tlight() else {
                            continue;
                        };
                        light_next.toggled_on_flag
                    };

                    let mut light_curr = components[index].borrow_mut();
                    let Some(light_curr) = light_curr.as_tlight_mut() else {
                        continue;
                    };
                    light_curr.message(
                        if next_status {
                            MessageCode::T_LIGHT_TURN_ON_TIMED
                        } else {
                            MessageCode::T_LIGHT_TURN_OFF_TIMED
                        },
                        0.0,
                        ctx,
                    )?;
                }
                {
                    let mut last_light = components[components.len() - 1].borrow_mut();
                    if let Some(last_light) = last_light.as_tlight_mut() {
                        last_light.message(
                            if first_status {
                                MessageCode::T_LIGHT_TURN_ON_TIMED
                            } else {
                                MessageCode::T_LIGHT_TURN_OFF_TIMED
                            },
                            0.0,
                            ctx,
                        )?;
                    }
                }
                self.reschedule_animation(value, ctx)?;
            }
            MessageCode::T_LIGHT_GROUP_LIGHT_SHOW_ANIMATION => {
                if self.animation_flag || self.message_field_2 == MessageCode::T_LIGHT_GROUP_NULL {
                    self.start_animation(ctx)?;
                }
                self.message_field_2 = code;
                self.animation_flag = false;
                let components = self.get_light_components();
                for comp in components {
                    if let Some(light_comp) = comp.borrow_mut().as_tlight_mut() {
                        if rand::random::<u32>() % 100 > 70 {
                            let rand_val = utils::rand_float_pb() * value * 3.0f32 + 0.1f32;
                            light_comp.message(
                                MessageCode::T_LIGHT_TURN_ON_TIMED,
                                rand_val,
                                ctx,
                            )?;
                        }
                    }
                }
                self.reschedule_animation(value, ctx)?;
            }
            MessageCode::T_LIGHT_GROUP_GAME_OVER_ANIMATION => {
                if self.animation_flag || self.message_field_2 == MessageCode::T_LIGHT_GROUP_NULL {
                    self.start_animation(ctx)?;
                }
                self.message_field_2 = code;
                self.animation_flag = false;
                let components = self.get_light_components();
                for comp in components {
                    if let Some(light_comp) = comp.borrow_mut().as_tlight_mut() {
                        let rand_val = rand::random::<u32>() % 100 > 70;
                        light_comp.message(
                            MessageCode::T_LIGHT_RESET_AND_TOGGLE_VALUE,
                            rand_val as i32 as f32,
                            ctx,
                        )?;
                    }
                }
                self.reschedule_animation(value, ctx)?;
            }
            MessageCode::T_LIGHT_GROUP_RANDOM_ANIMATION_SATURATION => {
                let mut no_bmp_ind_1_count = 0;
                let components = self.get_light_components();
                for comp in &components {
                    if let Some(light_comp) = comp.borrow().as_tlight() {
                        if !light_comp.light_on_flag {
                            no_bmp_ind_1_count += 1;
                        }
                    }
                }
                if no_bmp_ind_1_count == 0 {
                    return Ok(0);
                }

                let mut rand_mod_count = rand::random::<u32>() % no_bmp_ind_1_count as u32;
                for comp in components.iter().rev() {
                    let mut comp_borrow = comp.borrow_mut();
                    let Some(light_comp) = comp_borrow.as_tlight_mut() else {
                        continue;
                    };

                    if !light_comp.light_on_flag {
                        if rand_mod_count == 0 {
                            light_comp.message(MessageCode::T_LIGHT_TURN_ON, 0.0, ctx)?;
                            break;
                        }
                        rand_mod_count -= 1;
                    }
                }

                if self.message_field_2 != MessageCode::T_LIGHT_GROUP_NULL {
                    self.start_animation(ctx)?;
                }
            }
            MessageCode::T_LIGHT_GROUP_RANDOM_ANIMATION_DESATURATION => {
                let components = self.get_light_components();
                let mut bmp_ind_1_count = 0;
                for comp in &components {
                    if let Some(light_comp) = comp.borrow().as_tlight()
                        && light_comp.light_on_flag
                    {
                        bmp_ind_1_count += 1;
                    }
                }
                if bmp_ind_1_count == 0 {
                    return Ok(0);
                }

                let mut rand_mod_count = rand::random::<u32>() % bmp_ind_1_count as u32;
                for comp in components.iter().rev() {
                    let mut comp_borrow = comp.borrow_mut();
                    let Some(light_comp) = comp_borrow.as_tlight_mut() else {
                        continue;
                    };

                    if light_comp.light_on_flag {
                        if rand_mod_count == 0 {
                            light_comp.message(MessageCode::T_LIGHT_TURN_OFF, 0.0, ctx)?;
                            break;
                        }
                        rand_mod_count -= 1;
                    }
                }

                if self.message_field_2 != MessageCode::T_LIGHT_GROUP_NULL {
                    self.start_animation(ctx)?;
                }
            }
            MessageCode::T_LIGHT_GROUP_OFFSET_ANIMATION_FORWARD => {
                let index = self.next_light_up();
                if index < 0 {
                    return Ok(0);
                }
                let components = self.get_light_components();
                components[index as usize]
                    .borrow_mut()
                    .as_tlight_mut()
                    .unwrap()
                    .message(MessageCode::T_LIGHT_TURN_ON, 0.0, ctx)?;
                if self.message_field_2 != MessageCode::T_LIGHT_GROUP_NULL {
                    self.start_animation(ctx)?;
                }
                return Ok(1);
            }
            MessageCode::T_LIGHT_GROUP_OFFSET_ANIMATION_BACKWARD => {
                let index = self.next_light_down();
                if index < 0 {
                    return Ok(0);
                }
                let components = self.get_light_components();
                components[index as usize]
                    .borrow_mut()
                    .as_tlight_mut()
                    .unwrap()
                    .message(MessageCode::T_LIGHT_TURN_OFF, 0.0, ctx)?;
                if self.message_field_2 != MessageCode::T_LIGHT_GROUP_NULL {
                    self.start_animation(ctx)?;
                }
                return Ok(1);
            }
            MessageCode::T_LIGHT_GROUP_RESET => {
                if self.timer > 0 {
                    ctx.timer_manager.borrow_mut().kill_id(self.timer)?;
                }
                self.timer = 0;
                if self.message_field_2 == MessageCode::T_LIGHT_GROUP_ANIMATION_BACKWARD
                    || self.message_field_2 == MessageCode::T_LIGHT_GROUP_ANIMATION_FORWARD
                    || self.message_field_2 == MessageCode::T_LIGHT_GROUP_LIGHT_SHOW_ANIMATION
                {
                    self.message(MessageCode::T_LIGHT_RESET_TIMED, 0.0, ctx)?;
                }
                self.message_field_2 = MessageCode::T_LIGHT_GROUP_NULL;
                self.animation_flag = false;
            }
            MessageCode::T_LIGHT_GROUP_TURN_ON_AT_INDEX => {
                let index = value.floor() as i64;
                let components = self.get_light_components();
                if index >= components.len() as i64 || index < 0 {
                    return Ok(0);
                }

                let mut comp_borrow = components[index as usize].borrow_mut();
                let Some(light) = comp_borrow.as_tlight_mut() else {
                    return Ok(0);
                };
                light.message(MessageCode::T_LIGHT_TURN_ON, 0.0, ctx)?;
                if self.message_field_2 != MessageCode::T_LIGHT_GROUP_NULL {
                    self.start_animation(ctx)?;
                }
            }
            MessageCode::T_LIGHT_GROUP_TURN_OFF_AT_INDEX => {
                let index = value.floor() as i64;
                let components = self.get_light_components();
                if index >= components.len() as i64 || index < 0 {
                    return Ok(0);
                }

                let mut comp_borrow = components[index as usize].borrow_mut();
                let Some(light) = comp_borrow.as_tlight_mut() else {
                    return Ok(0);
                };
                light.message(MessageCode::T_LIGHT_TURN_OFF, 0.0, ctx)?;
                if self.message_field_2 != MessageCode::T_LIGHT_GROUP_NULL {
                    self.start_animation(ctx)?;
                }
            }
            MessageCode::T_LIGHT_GROUP_GET_ON_COUNT => {
                let mut bmp_1_count = 0;
                let components = self.get_light_components();
                for comp in components {
                    if let Some(light_comp) = comp.borrow().as_tlight() {
                        if light_comp.light_on_flag {
                            bmp_1_count += 1;
                        }
                    }
                }
                return Ok(bmp_1_count);
            }
            MessageCode::T_LIGHT_GROUP_GET_LIGHT_COUNT => {
                return Ok(self.get_light_components().len() as i32);
            }
            MessageCode::T_LIGHT_GROUP_GET_MESSAGE2 => {
                return Ok(self.message_field_2.0);
            }
            MessageCode::T_LIGHT_GROUP_GET_ANIMATION_FLAG => {
                return Ok(self.animation_flag as i32);
            }
            MessageCode::T_LIGHT_GROUP_RESET_AND_TURN_ON => {
                let index = self.next_light_up();
                if index < 0 {
                    return Ok(0);
                }

                if self.message_field_2 != MessageCode::T_LIGHT_GROUP_NULL || self.animation_flag {
                    self.message(MessageCode::T_LIGHT_GROUP_RESET, 0.0, ctx)?;
                }

                let Some(component) = self.get_light_component_by_position(index as usize) else {
                    return Ok(0);
                };
                let mut comp_borrow = component.borrow_mut();
                let Some(light) = comp_borrow.as_tlight_mut() else {
                    return Ok(0);
                };
                light.message(
                    MessageCode::T_LIGHT_FLASHER_START_TIMED_THEN_STAY_ON,
                    value,
                    ctx,
                )?;
                return Ok(1);
            }
            MessageCode::T_LIGHT_GROUP_RESET_AND_TURN_OFF => {
                let index = self.next_light_down();
                if index < 0 {
                    return Ok(0);
                }

                if self.message_field_2 != MessageCode::T_LIGHT_GROUP_NULL || self.animation_flag {
                    self.message(MessageCode::T_LIGHT_GROUP_RESET, 0.0, ctx)?;
                }

                let Some(component) = self.get_light_component_by_position(index as usize) else {
                    return Ok(0);
                };
                let mut comp_borrow = component.borrow_mut();
                let Some(light) = comp_borrow.as_tlight_mut() else {
                    return Ok(0);
                };
                light.message(
                    MessageCode::T_LIGHT_FLASHER_START_TIMED_THEN_STAY_OFF,
                    value,
                    ctx,
                )?;
                return Ok(1);
            }
            MessageCode::T_LIGHT_GROUP_RESTART_NOTIFY_TIMER => {
                if self.notify_timer > 0 {
                    ctx.timer_manager.borrow_mut().kill_id(self.notify_timer)?;
                }
                self.notify_timer = 0;

                if value > 0.0 {
                    self.notify_timer =
                        ctx.set_timer(value, &raw mut *self as *mut c_void, notify_timer_expired)?;
                }
            }
            MessageCode::T_LIGHT_GROUP_FLASH_WHEN_ON => {
                let components = self.get_light_components();
                for comp in components.iter().rev() {
                    let mut comp_borrow = comp.borrow_mut();
                    let Some(light) = comp_borrow.as_tlight_mut() else {
                        continue;
                    };

                    if light.light_on_flag {
                        light.message(MessageCode::T_LIGHT_TURN_OFF, 0.0, ctx)?;
                        light.message(
                            MessageCode::T_LIGHT_FLASHER_START_TIMED_THEN_STAY_OFF,
                            value,
                            ctx,
                        )?;
                    }
                }
            }
            MessageCode::T_LIGHT_GROUP_TOGGLE_SPLIT_INDEX => {
                self.dispatch_control(code, ctx)?;

                let index = value.floor() as i64;
                let components = self.get_light_components();
                let count = components.len();
                if index < 0 || index >= count as i64 {
                    return Ok(0);
                }
                let index = index as usize;

                for i in ((index + 1)..count).rev() {
                    let mut comp_borrow = components[i].borrow_mut();
                    let Some(light) = comp_borrow.as_tlight_mut() else {
                        continue;
                    };
                    light.message(MessageCode::T_LIGHT_RESET_AND_TURN_OFF, 0.0, ctx)?;
                }

                for i in (0..=index).rev() {
                    let mut comp_borrow = components[i].borrow_mut();
                    let Some(light) = comp_borrow.as_tlight_mut() else {
                        continue;
                    };
                    light.message(MessageCode::T_LIGHT_RESET_AND_TURN_ON, 0.0, ctx)?;
                }
            }
            MessageCode::T_LIGHT_GROUP_START_FLASHER => {
                let index = self.next_light_down();
                if index < 0 {
                    return Ok(0);
                }

                let Some(component) = self.get_light_component_by_position(index as usize) else {
                    return Ok(0);
                };
                let mut comp_borrow = component.borrow_mut();
                let Some(light) = comp_borrow.as_tlight_mut() else {
                    return Ok(0);
                };
                light.message(MessageCode::T_LIGHT_FLASHER_START, 0.0, ctx)?;
            }

            _ => {
                let components = self.get_light_components();
                for comp in components.iter().rev() {
                    let mut comp_borrow = comp.borrow_mut();
                    if let Some(light) = comp_borrow.as_tlight_mut() {
                        light.message(code, value, ctx)?;
                    }
                }
            }
        }

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
            list: vec![],
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
                if let Some(table_rc) = instance.base.pinball_table.as_ref().and_then(|t| t.upgrade())
                    && let Some(component) = table_rc.borrow().find_component(light_idx)
                        && component.borrow().as_tlight().is_some()
                    {
                        instance.list.push(component);
                    }
                unsafe { group_idx_ptr = group_idx_ptr.add(1) }
            }
        }

        Ok(instance)
    }
}
