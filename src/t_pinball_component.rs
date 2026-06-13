use crate::maths::*;
use crate::message_code::MessageCode;
use crate::render::RenderSprite;
use crate::{control::ComponentControl, loader, loader::VisualStruct};
use crate::{loader::SpriteData, t_pinball_table::TPinballTable};
use std::ffi::{CStr, CString};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

#[derive(Default)]
pub struct TPinballComponent {
    pub unused_base_flag: Rc<Cell<bool>>,
    pub active_flag: Rc<Cell<bool>>,
    pub message_field: MessageCode,
    pub group_name: Option<Rc<Cell<String>>>,
    pub control: Option<Weak<RefCell<ComponentControl>>>,
    pub group_index: i32,
    pub render_sprite: RenderSprite,
    pub pinball_table: Option<Weak<RefCell<TPinballTable>>>,
    pub list_bitmap: Vec<SpriteData>,

    visual_pos_norm_x: f32,
    visual_pos_norm_y: f32,
}

pub trait IPinballComponent {
    fn sprite_set(&self, index: i32);
    fn sprite_set_ball(&self, index: i32, pos: Vector2i, depth: f32);
    fn get_coordinates(&self) -> Vector2;
    fn get_scoring(&self, index: u32) -> i32;
    fn port_draw(&self);
    fn message(&mut self, code: MessageCode, value: f32) -> MessageCode;
}

impl TPinballComponent {
    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        load_visuals: bool,
    ) -> Self {
        let visual: VisualStruct;

        let mut instance = Self {
            unused_base_flag: Rc::new(Cell::new(false)),
            active_flag: Rc::new(Cell::new(false)),
            message_field: MessageCode(0),
            group_name: Some(Rc::new(Cell::new(String::new()))),
            control: None,
            group_index,
            render_sprite: RenderSprite::default(),
            pinball_table: None,
            list_bitmap: Vec::new(),
            visual_pos_norm_x: -1.0,
            visual_pos_norm_y: -1.0,
        };

        if let Some(table_result) = table {
            instance.pinball_table = Some(table_result)
            // TODO: table.component_list.push_back();
        }

        if group_index >= 0 {
            let name = loader::query_name(group_index).unwrap();
            let name_string = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
            instance.group_name = Some(Rc::new(Cell::new(name_string)))
        }

        if load_visuals && group_index >= 0 {
            // TODO: Create module
            //let visual_count = loader::query_visual_states(group_index);
            // TODO: For loop L#33...
        }

        instance
    }
}

impl Drop for TPinballComponent {
    fn drop(&mut self) {
        if self.pinball_table.is_some() {
            // TODO: Add field, use let Some, upgrade weakptr, borrow mut?
            //let components = self.pinball_table.unwrap().component_list;
            //TODO: Implement component list first then let position = // std::find begin end for this
            // Continue from L94
        }

        // TODO: We don't need this
        //drop(self.list_bitmap); // TODO: Does list_bitmap implement drop?
    }
}

impl IPinballComponent for TPinballComponent {
    fn sprite_set(&self, index: i32) {
        todo!()
    }

    fn sprite_set_ball(&self, index: i32, pos: Vector2i, depth: f32) {
        todo!()
    }

    fn get_coordinates(&self) -> Vector2 {
        Vector2 {
            x: self.visual_pos_norm_x,
            y: self.visual_pos_norm_y,
        }
    }

    fn get_scoring(&self, index: u32) -> i32 {
        todo!()
    }

    fn port_draw(&self) {
        // Doesn't have an impl
    }

    fn message(&mut self, code: MessageCode, value: f32) -> MessageCode {
        self.message_field = code;
        if code == MessageCode::RESET {
            self.message_field = MessageCode(0);
        }

        MessageCode(0)
    }
}
