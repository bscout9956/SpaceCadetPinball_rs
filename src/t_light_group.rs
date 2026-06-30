use crate::errors::LoaderError;
use crate::loader;
use crate::maths::{Vector2, Vector2i};
use crate::message_code::MessageCode;
use crate::state::pinball_state::PinballState;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Weak;
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

impl IPinballComponent for TLightGroup {
    fn render_sprite(&self) -> Option<&RenderSprite> {
        self.base.render_sprite.as_ref()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn group_name(&self) -> Option<String> {
        match self.base.group_name.as_ref() {
            None => None,
            Some(str) => Some(str.borrow().to_string()),
        }
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
        todo!()
    }

    fn set_active_flag(&mut self, active: bool) {
        self.base.set_active_flag(active);
    }
}

#[derive(Error, Debug)]
pub enum TLightGroupError {
    #[error("Error loading data `{0}`")]
    LoaderError(#[from] LoaderError),
}

use crate::render::RenderSprite;
use anyhow::Result;

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
            message_field_2: Default::default(),
            animation_flag: false,
            notify_timer: 0,
            timer: 0,
            player_data: std::array::from_fn(|_| TLightGroupPlayerBackup::default()),
        };

        if group_index > 0 {
            let mut count = 0;
            // TODO: Remove unwrap and deref
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
