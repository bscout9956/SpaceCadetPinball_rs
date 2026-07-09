use crate::context::component_context::ComponentContext;
use crate::control::ComponentControl;
use crate::loader;
use crate::maths::Vector2;
use crate::message_code::MessageCode;
use crate::render::RenderSpriteRef;
use crate::state::pinball_state::PinballState;
use crate::t_edge_manager::TEdgeManager;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use anyhow::Result;
use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Default)]
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
        _code: MessageCode,
        _value: f32,
        _component_context: &mut ComponentContext,
    ) -> Result<i32> {
        println!("Beep boop, flash");
        //TODO finish me
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
