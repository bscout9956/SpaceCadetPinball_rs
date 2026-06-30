use crate::maths::{Vector2, Vector2i};
use crate::message_code::MessageCode;
use crate::render::RenderSprite;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use std::any::Any;

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

    fn group_name(&self) -> Option<String> {
        if let Some(name) = self.base.group_name.as_ref() {
            let name_str = name.borrow().to_string();
            return Some(name_str);
        }
        None
    }

    fn group_index(&self) -> i32 {
        self.base.group_index
    }
    fn sprite_set(&mut self, index: i32) {
        todo!()
    }

    fn sprite_set_ball(&self, index: i32, pos: Vector2i, depth: f32) {
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

    fn message(&mut self, code: MessageCode, value: f32) -> i32 {
        println!("Beep boop, flash");
        //TODO finish me
        0
    }

    fn set_active_flag(&mut self, active: bool) {
        self.base.active_flag.set(active);
    }
}
