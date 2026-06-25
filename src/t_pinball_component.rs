use crate::maths::*;
use crate::message_code::MessageCode;
use crate::render::RenderSprite;
use crate::state::loader_state::LoaderState;
use crate::{control::ComponentControl, loader, loader::VisualStruct};
use crate::{loader::SpriteData, t_pinball_table::TPinballTable};
use std::any::Any;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

#[derive(Default, Clone)]
pub struct TPinballComponent {
    pub unused_base_flag: Rc<Cell<bool>>,
    pub active_flag: Rc<Cell<bool>>,
    pub message_field: MessageCode,
    pub group_name: Option<Rc<RefCell<String>>>,
    pub control: Option<Weak<RefCell<ComponentControl>>>,
    pub group_index: i32,
    pub render_sprite: Option<RenderSprite>,
    pub pinball_table: Option<Weak<RefCell<TPinballTable>>>,
    pub list_bitmap: Vec<SpriteData>,

    visual_pos_norm_x: f32,
    visual_pos_norm_y: f32,
}

pub trait IPinballComponent {
    fn as_any(&self) -> &dyn Any;
    fn group_name(&self) -> Option<String>;
    fn group_index(&self) -> i32;
    fn sprite_set(&mut self, index: i32);
    fn sprite_set_ball(&self, index: i32, pos: Vector2i, depth: f32);
    fn get_coordinates(&self) -> Vector2;
    fn get_scoring(&self, index: u32) -> i32;
    fn port_draw(&self);
    fn message(&mut self, code: MessageCode, value: f32) -> i32;
}

impl TPinballComponent {
    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        load_visuals: bool,
        loader_state: &mut LoaderState,
    ) -> Self {
        let visual: VisualStruct;

        let mut instance = Self {
            unused_base_flag: Rc::new(Cell::new(false)),
            active_flag: Rc::new(Cell::new(false)),
            message_field: MessageCode(0),
            group_name: Some(Rc::new(RefCell::new(String::new()))),
            control: None,
            group_index,
            render_sprite: None,
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
            let name = loader::query_name(group_index, loader_state).unwrap();
            instance.group_name = Some(Rc::new(RefCell::new(name)))
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
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn group_name(&self) -> Option<String> {
        if let Some(group_name) = self.group_name.as_ref() {
            Some(group_name.borrow().clone())
        } else {
            None
        }
    }

    fn group_index(&self) -> i32 {
        self.group_index
    }
    fn sprite_set(&mut self, index: i32) {
        if self.list_bitmap.is_empty() {
            return;
        }

        let mut x_pos = 0;
        let mut y_pos = 0;
        let mut bmp = None;
        let mut zmap = None;

        if index >= 0
            && let Some(sprite_data) = self.list_bitmap.get(index as usize)
        {
            bmp = sprite_data.bmp.clone();
            zmap = sprite_data.zmap.clone();

            if let Some(ref b) = bmp
                && let Some(table_weak) = &self.pinball_table
                && let Some(table_rc) = table_weak.upgrade()
            {
                let table_borrow = table_rc.borrow();
                x_pos = b.x_position - table_borrow.x_offset;
                y_pos = b.y_position - table_borrow.y_offset;
            }
        } else {
            bmp = None;
            zmap = None;
            if let Some(rs) = self.render_sprite.as_ref() {
                x_pos = rs.bmp_rect.x_position;
                y_pos = rs.bmp_rect.y_position;
            } else {
                eprintln!("WARNING! Missing render sprite");
            }
        }

        if let Some(rs) = self.render_sprite.as_mut() {
            rs.set(bmp, zmap, x_pos, y_pos);
        }
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
        // TODO: Doesn't have an impl?
    }

    fn message(&mut self, code: MessageCode, value: f32) -> i32 {
        // TODO?
        self.message_field = code;
        if code == MessageCode::RESET {
            self.message_field = MessageCode(0);
        }

        0
    }
}
