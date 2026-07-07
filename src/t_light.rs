use crate::context::component_context::ComponentContext;
use crate::maths::Vector2;
use crate::message_code::MessageCode;
use crate::render::RenderSprite;
use crate::t_edge_manager::TEdgeManager;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use anyhow::Result;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

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

impl IPinballComponent for TLight {
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
        component_context: &mut ComponentContext,
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
}
