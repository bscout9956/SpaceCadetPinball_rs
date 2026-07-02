use crate::maths::*;
use crate::message_code::MessageCode;
use crate::render::{RenderSprite, VisualTypes};
use crate::{control::ComponentControl, loader, loader::VisualStruct};
use crate::{loader::SpriteData, t_pinball_table::TPinballTable};
use anyhow::Context;
use std::any::Any;
use std::ops::Index;
use std::sync::Arc;
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
    fn render_sprite(&self) -> Option<&RenderSprite>;
    fn as_any(&self) -> &dyn Any;
    fn group_name(&self) -> Option<Rc<RefCell<String>>>;
    fn group_index(&self) -> i32;
    fn sprite_set(&mut self, index: i32);
    fn sprite_set_ball(&self, index: i32, pos: Vector2i, depth: f32);
    fn get_coordinates(&self) -> Vector2;
    fn get_scoring(&self, index: u32) -> i32;
    fn port_draw(&self);
    fn message(&mut self, code: MessageCode, value: f32, time_ticks: usize) -> i32;
    fn set_active_flag(&mut self, active: bool);
}

use crate::state::pinball_state::PinballState;
use anyhow::Result;

impl TPinballComponent {
    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        load_visuals: bool,
        state: &mut PinballState,
    ) -> Result<Self> {
        let mut visual: VisualStruct = Default::default();

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
            // TODO: IMPORTANT: PLEASE: table.component_list.push_back();
        }

        if group_index >= 0 {
            let name = loader::query_name(group_index, &mut state.loader_state)
                .context("Failed to query name for TPinballComponent")?;
            instance.group_name = Some(Rc::new(RefCell::new(name)))
        }

        if load_visuals && group_index >= 0 {
            let visual_count = loader::query_visual_states(group_index, &mut state.loader_state)
                .context("Failed to query visual count for TPinballComponent")?;
            for index in 0..visual_count {
                loader::query_visual(group_index, index as i32, &mut visual, state)
                    .context("Failed to query visual for init on TPinballComponent")?;

                if visual.bitmap.bmp.as_ref().is_some() {
                    if visual.bitmap.zmap.as_ref().is_none() {
                        panic!("Bitmap/zMap pairing is mandatory");
                    }
                    if instance.list_bitmap.is_empty() {
                        instance.list_bitmap = Vec::new();
                    }
                    instance.list_bitmap.push(visual.bitmap.clone());
                }
            }

            if !instance.list_bitmap.is_empty() {
                let mut bmp_1_rect: RectangleType = Default::default();
                let mut tmp_rect: RectangleType = Default::default();

                let root_sprite = instance.list_bitmap.get(0);

                if let Some(rs) = root_sprite {
                    if let Some(root_bmp) = (*rs.bmp).as_ref()
                        && let Some(t) = instance.pinball_table.as_ref()
                    {
                        let table = t.upgrade().unwrap();
                        bmp_1_rect.x_position = root_bmp.x_position - table.borrow().x_offset;
                        bmp_1_rect.y_position = root_bmp.y_position - table.borrow().y_offset;
                        bmp_1_rect.width = root_bmp.width;
                        bmp_1_rect.height = root_bmp.height;

                        for index in 1..instance.list_bitmap.len() {
                            let bmp_opt = instance.list_bitmap.index(index).bmp.clone();
                            if let Some(bmp) = (*bmp_opt).as_ref() {
                                tmp_rect.x_position = bmp.x_position - table.borrow().x_offset;
                                tmp_rect.y_position = bmp.y_position - table.borrow().y_offset;
                                tmp_rect.width = bmp.width;
                                tmp_rect.height = bmp.height;
                                let mut copy_rect = RectangleType::default();
                                enclosing_box(&bmp_1_rect, &tmp_rect, &mut copy_rect);
                                bmp_1_rect = copy_rect;
                            }
                        }

                        instance.render_sprite = Some(RenderSprite::new(
                            VisualTypes::Sprite,
                            Arc::new(Some(root_bmp.clone())),
                            root_sprite.unwrap().zmap.clone(),
                            root_bmp.x_position - table.borrow().x_offset,
                            root_bmp.y_position - table.borrow().y_offset,
                            Some(bmp_1_rect),
                            &mut state.render_state,
                        ));

                        //TODO: Continue from here, l76 of cpp
                    }
                }
            }
        }

        Ok(instance)
    }
}

impl Drop for TPinballComponent {
    fn drop(&mut self) {
        if self.pinball_table.is_some() {
            // TODO: Add field, use let Some, upgrade weak ptr, borrow mut?
            //let components = self.pinball_table.unwrap().component_list;
            //TODO: Implement component list first then let position = // std::find begin end for this
            // Continue from L94
        }

        // TODO: We don't need this
        //drop(self.list_bitmap); // TODO: Does list_bitmap implement drop?
    }
}

impl IPinballComponent for TPinballComponent {
    fn render_sprite(&self) -> Option<&RenderSprite> {
        self.render_sprite.as_ref()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn group_name(&self) -> Option<Rc<RefCell<String>>> {
        self.group_name.clone()
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
        let bmp;
        let zmap;

        if index >= 0
            && let Some(sprite_data) = self.list_bitmap.get(index as usize)
        {
            bmp = sprite_data.bmp.clone();
            zmap = sprite_data.zmap.clone();

            if let Some(ref b) = *bmp
                && let Some(table_weak) = &self.pinball_table
                && let Some(table_rc) = table_weak.upgrade()
            {
                let table_borrow = table_rc.borrow();
                x_pos = b.x_position - table_borrow.x_offset;
                y_pos = b.y_position - table_borrow.y_offset;
            }
        } else {
            bmp = Arc::new(None);
            zmap = Arc::new(None);
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

    #[allow(unused)]
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

    #[allow(unused)]
    fn message(&mut self, code: MessageCode, value: f32, time_ticks: usize) -> i32 {
        // TODO?
        self.message_field = code;
        if code == MessageCode::RESET {
            self.message_field = MessageCode(0);
        }

        0
    }

    fn set_active_flag(&mut self, active: bool) {
        self.active_flag.set(active);
    }
}
