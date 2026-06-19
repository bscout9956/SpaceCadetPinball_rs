use crate::loader;
use crate::maths::{Vector2, Vector2i};
use crate::message_code::MessageCode;
use crate::state::loader_state::LoaderState;
use crate::t_light::TLight;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use std::cell::RefCell;
use std::rc::Weak;

#[derive(Default)]
struct TLightGroupPlayerBackup {
    message_field: i32,
    message_field_2: MessageCode,
    timer1time: f32,
    unknown3: i32,
}

pub struct TLightGroup {
    base: TPinballComponent,
    list: Vec<Option<TLight>>,
    timer1time: f32,
    timer1time_default: f32,
    message_field_2: MessageCode,
    animation_flag: bool,
    notify_timer: i32,
    timer: i32,
    player_data: [TLightGroupPlayerBackup; 4],
}

impl IPinballComponent for TLightGroup {
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

    fn message(&mut self, code: MessageCode, value: f32) -> MessageCode {
        todo!()
    }
}

// TODO: Verify I'm finished
impl TLightGroup {
    pub(crate) fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        loader_state: &mut LoaderState,
    ) -> TLightGroup {
        let base = TPinballComponent::new(table, group_index, false, loader_state);

        let mut instance = Self {
            base,
            list: vec![],
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
            let timer_1_time_default =
                loader::query_float_attribute_ptr(group_index, 0, 903, loader_state);
            let group_idx =
                loader::query_int_attribute(group_index, 1027, &raw mut count, loader_state).unwrap();
            for idx in 0..count {
                if let Some(t) = table.as_ref() {
                    let upgrade_table = t.upgrade().unwrap();
                    let tble = upgrade_table.borrow();
                    let comp = (*tble).find_component(group_idx)
                    
                }
                group_idx += 1;
            }
        }
    }
}
