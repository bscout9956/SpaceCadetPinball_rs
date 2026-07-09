use crate::context::component_context::ComponentContext;
use crate::control::ComponentControl;
use crate::maths::Vector2;
use crate::message_code::MessageCode;
use crate::render::RenderSpriteRef;
use crate::state::pinball_state::PinballState;
use crate::t_edge_manager::TEdgeManager;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use crate::{loader, utils};
use anyhow::Result;
use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::{Rc, Weak};

#[derive(Clone, Copy, Default)]
pub struct TlightPlayerBackup {
    message_field: i32,
    light_on_flag: bool,
    light_on_bmp_index: i32,
    flasher_on_flag: bool,
}

pub struct TLight {
    base: TPinballComponent,
    pub bmp_arr: [i32; 2], // init'd to -1?
    pub flash_delay: [f32; 2],
    pub flash_timer: i32,
    pub flash_light_on_flag: bool,
    pub light_on_flag: bool,
    pub flasher_on_flag: bool,
    pub toggled_off_flag: bool,
    pub toggled_on_flag: bool,
    pub turn_off_after_flashing_fg: bool,
    pub light_on_bmp_index: i32,
    pub source_delay: [f32; 2],
    pub timeout_timer: i32,
    pub undo_override_timer: i32,
    pub temporary_override_flag: bool,
    pub previous_bitmap: i32, // init to -1,
    pub player_data: [TlightPlayerBackup; 4],
}

impl TLight {
    fn flasher_start(&mut self, bmp_index: bool, ctx: &mut ComponentContext) -> Result<()> {
        self.flash_light_on_flag = bmp_index;
        unsafe { flasher_callback(0, &raw mut *self as *mut c_void, ctx)? };
        Ok(())
    }
}

unsafe extern "C" fn flasher_callback(
    _timer_id: i32,
    caller: *mut c_void,
    ctx: &mut ComponentContext,
) -> Result<()> {
    let light = caller as *mut TLight;
    unsafe {
        (*light).flash_light_on_flag ^= true;
        (*light).set_sprite_bmp((*light).bmp_arr[(*light).flash_light_on_flag as usize])?;
        (*light).flash_timer = ctx.set_timer(
            (*light).flash_delay[(*light).flash_light_on_flag as usize],
            caller,
            flasher_callback,
        )?;
    }
    Ok(())
}

impl TLight {
    pub(crate) fn schedule_timeout(&mut self, time: f32, ctx: &mut ComponentContext) -> Result<()> {
        self.flash_delay = self.source_delay;
        if self.timeout_timer > 0 {
            ctx.timer_manager.borrow_mut().kill_id(self.timeout_timer)?;
        }
        self.timeout_timer = 0;
        if time > 0.0 {
            self.timeout_timer = ctx.timer_manager.borrow_mut().set(
                time,
                &raw mut *self as *mut c_void,
                timer_expired,
                ctx,
            )?;
        }
        Ok(())
    }
}

unsafe extern "C" fn timer_expired(
    _timer_id: i32,
    caller: *mut c_void,
    ctx: &mut ComponentContext,
) -> Result<()> {
    let light = caller as *mut TLight;
    unsafe {
        if (*light).flasher_on_flag {
            (*light).flasher_stop(-1, ctx)?;
        }
        (*light).set_sprite_bmp((*light).bmp_arr[(*light).light_on_flag as usize])?;
        (*light).toggled_off_flag = false;
        (*light).toggled_on_flag = false;
        (*light).flasher_on_flag = false;
        (*light).timeout_timer = 0;

        if (*light).turn_off_after_flashing_fg {
            (*light).turn_off_after_flashing_fg = false;
            (*light).message(MessageCode::T_LIGHT_RESET_AND_TURN_OFF, 0.0, ctx)?;
        }

        if let Some(c) = (*light).base.control.as_mut() {
            let control = c.upgrade().unwrap();
            // TODO: Complex logic, implement me though, otherwise the game will not respond
            // control
            //     .borrow()
            //     .handler(MessageCode::CONTROL_TIMER_EXPIRED, (*light));
        }
    }

    Ok(())
}

impl TLight {
    pub(crate) fn reset(&mut self, ctx: &mut ComponentContext) -> Result<()> {
        if self.timeout_timer > 0 {
            ctx.timer_manager.borrow_mut().kill_id(self.timeout_timer)?;
        }
        if self.undo_override_timer > 0 {
            ctx.timer_manager
                .borrow_mut()
                .kill_id(self.undo_override_timer)?;
        }
        if self.flasher_on_flag {
            self.flasher_stop(-1, ctx)?;
        }
        self.timeout_timer = 0;
        self.undo_override_timer = 0;
        self.light_on_flag = false;
        self.light_on_bmp_index = 0;
        self.toggled_off_flag = false;
        self.toggled_on_flag = false;
        self.flasher_on_flag = false;
        self.temporary_override_flag = false;
        self.turn_off_after_flashing_fg = false;
        self.previous_bitmap = -1;
        self.bmp_arr[0] = -1;
        self.bmp_arr[1] = 0;
        self.set_sprite_bmp(self.bmp_arr[0])?;
        self.set_message_field(MessageCode(0));
        Ok(())
    }

    fn flasher_stop(&mut self, bmp_index: i32, ctx: &mut ComponentContext) -> Result<()> {
        if self.flash_timer > 0 {
            ctx.timer_manager.borrow_mut().kill_id(self.flash_timer)?;
        }
        self.flash_timer = 0;
        if bmp_index >= 0 {
            self.flash_light_on_flag = bool::try_from(bmp_index)?;
            self.set_sprite_bmp(self.bmp_arr[self.flash_light_on_flag as usize])?;
        }
        Ok(())
    }

    fn set_sprite_bmp(&mut self, index: i32) -> Result<()> {
        self.previous_bitmap = index;
        if !self.temporary_override_flag {
            self.sprite_set(index);
        }
        Ok(())
    }

    fn player_attrs(&self) -> TlightPlayerBackup {
        TlightPlayerBackup {
            flasher_on_flag: self.flasher_on_flag,
            light_on_bmp_index: self.light_on_bmp_index,
            light_on_flag: self.light_on_flag,
            message_field: self.message_field().0,
        }
    }
}

impl TLight {
    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<Self> {
        let base = TPinballComponent::new(table, group_index, true, state)?;
        let flash_delay_0 = unsafe {
            *loader::query_float_attribute_ptr(group_index, 0, 900, &mut state.loader_state)?
        };
        let flash_delay_1 = unsafe {
            *loader::query_float_attribute_ptr(group_index, 0, 901, &mut state.loader_state)?
        };

        Ok(Self {
            base,
            bmp_arr: [-1, 0],
            flash_delay: [flash_delay_0, flash_delay_1],
            flash_timer: 0,
            flash_light_on_flag: false,
            light_on_flag: false,
            flasher_on_flag: false,
            toggled_off_flag: false,
            toggled_on_flag: false,
            turn_off_after_flashing_fg: false,
            light_on_bmp_index: 0,
            source_delay: [flash_delay_0, flash_delay_1],
            timeout_timer: 0,
            undo_override_timer: 0,
            temporary_override_flag: false,
            previous_bitmap: -1,
            player_data: std::array::from_fn(|_| TlightPlayerBackup::default()),
        })
    }

    pub fn message_field(&self) -> MessageCode {
        self.base.message_field
    }

    pub fn set_message_field(&mut self, value: MessageCode) {
        self.base.message_field = value;
    }
}

impl IPinballComponent for TLight {
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

    fn get_coordinates(&self, tedge_manager: &TEdgeManager) -> Vector2 {
        self.base.get_coordinates(tedge_manager)
    }

    fn get_scoring(&self, index: u32) -> i32 {
        self.base.get_scoring(index)
    }

    fn port_draw(&self) {
        self.base.port_draw()
    }

    fn message(
        &mut self,
        code: MessageCode,
        value: f32,
        ctx: &mut ComponentContext,
    ) -> Result<i32> {
        let mut bmp_index = 0;

        match code {
            MessageCode::RESET => {
                self.reset(ctx)?;
                let player_attrs = self.player_attrs();
                for player in &mut self.player_data {
                    *player = player_attrs;
                }
            }
            MessageCode::PLAYER_CHANGED => {
                let current_player = {
                    let Some(t) = ctx.main_table.as_ref() else {
                        return Ok(0);
                    };

                    t.borrow().current_player as usize
                };
                self.player_data[current_player] = self.player_attrs();

                self.reset(ctx)?;

                let player = &self.player_data[value.floor() as usize];
                self.flasher_on_flag = player.flasher_on_flag;
                self.light_on_bmp_index = player.light_on_bmp_index;
                self.light_on_flag = player.light_on_flag;
                let message_field = player.message_field;
                self.set_message_field(MessageCode(message_field));
                if self.light_on_bmp_index > 0 {
                    self.message(
                        MessageCode::T_LIGHT_SET_ON_STATE_BMP_INDEX,
                        self.light_on_bmp_index as f32,
                        ctx,
                    )?;
                }
                if self.light_on_flag {
                    self.message(MessageCode::T_LIGHT_TURN_ON, 0.0, ctx)?;
                }
                if self.flasher_on_flag {
                    self.message(MessageCode::T_LIGHT_FLASHER_START, 0.0, ctx)?;
                }
            }
            MessageCode::T_LIGHT_TURN_OFF => {
                self.light_on_flag = false;
                if !self.flasher_on_flag && !self.toggled_off_flag && !self.toggled_on_flag {
                    self.set_sprite_bmp(self.bmp_arr[0])?;
                }
            }
            MessageCode::T_LIGHT_TURN_ON => {
                self.light_on_flag = true;
                if !self.flasher_on_flag && !self.toggled_off_flag && !self.toggled_on_flag {
                    self.set_sprite_bmp(self.bmp_arr[1])?;
                }
            }
            MessageCode::T_LIGHT_GET_LIGHT_ON_FLAG => {
                return Ok(self.light_on_flag as i32);
            }
            MessageCode::T_LIGHT_GET_FLASHER_ON_FLAG => {
                return Ok(self.light_on_flag as i32);
            }
            MessageCode::T_LIGHT_FLASHER_START => {
                self.schedule_timeout(0.0f32, ctx)?;
                if !self.flasher_on_flag || self.flash_timer == 0 {
                    self.flasher_on_flag = true;
                    self.toggled_off_flag = false;
                    self.toggled_on_flag = false;
                    self.turn_off_after_flashing_fg = false;
                    self.flasher_start(self.light_on_flag, ctx)?;
                }
            }
            MessageCode::T_LIGHT_APPLY_MULT_DELAY => {
                self.flash_delay[0] = value * self.source_delay[0];
                self.flash_delay[1] = value * self.source_delay[1];
            }
            MessageCode::T_LIGHT_APPLY_DELAY => {
                self.flash_delay = self.source_delay;
            }
            MessageCode::T_LIGHT_FLASHER_START_TIMED => {
                if !self.flasher_on_flag {
                    self.flasher_start(self.light_on_flag, ctx)?;
                }
                self.flasher_on_flag = true;
                self.toggled_off_flag = false;
                self.turn_off_after_flashing_fg = false;
                self.toggled_on_flag = false;
                self.schedule_timeout(value, ctx)?;
            }
            MessageCode::T_LIGHT_TURN_OFF_TIMED => {
                if !self.toggled_off_flag {
                    if self.flasher_on_flag {
                        self.flasher_stop(0, ctx)?;
                        self.flasher_on_flag = false;
                    } else {
                        self.set_sprite_bmp(self.bmp_arr[0])?;
                    }
                    self.toggled_off_flag = true;
                    self.toggled_on_flag = false;
                }
                self.schedule_timeout(value, ctx)?;
            }
            MessageCode::T_LIGHT_TURN_ON_TIMED => {
                if !self.toggled_on_flag {
                    if self.flasher_on_flag {
                        self.flasher_stop(1, ctx)?;
                        self.flasher_on_flag = false;
                    } else {
                        self.set_sprite_bmp(self.bmp_arr[1])?;
                    }
                    self.toggled_on_flag = true;
                    self.toggled_off_flag = false;
                }
                self.schedule_timeout(value, ctx)?;
            }
            MessageCode::T_LIGHT_SET_ON_STATE_BMP_INDEX => {
                self.light_on_bmp_index = utils::clamp(
                    &(value.floor() as i32),
                    &0,
                    &((self.base.list_bitmap.len() - 1) as i32),
                );
                self.bmp_arr[0] = -1;
                self.bmp_arr[1] = self.light_on_bmp_index;
                if !self.flasher_on_flag {
                    if self.toggled_off_flag {
                        bmp_index = 0;
                    } else if self.toggled_on_flag {
                        bmp_index = 1;
                    } else {
                        bmp_index = self.light_on_flag as i32;
                    }
                } else {
                    bmp_index = self.flash_light_on_flag as i32;
                }
                self.set_sprite_bmp(self.bmp_arr[bmp_index as usize])?;
            }
            // TODO: Keep going
            _ => {
                println!("Unknown message received: {:?}", code);
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

    fn set_control(&mut self, control: Option<Weak<RefCell<ComponentControl>>>) {
        self.base.set_control(control);
    }

    fn as_tlight(&self) -> Option<&TLight> {
        Some(self)
    }

    fn as_tlight_mut(&mut self) -> Option<&mut TLight> {
        Some(self)
    }
}
