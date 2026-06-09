use crate::message_code::MessageCode;
use crate::t_light::TLight;

struct TLightGroupPlayerBackup {
    message_field: i32,
    message_field_2: MessageCode,
    timer1time: f32,
    unknown3: i32,
}

pub struct TLightGroup {
    list: Vec<Option<TLight>>,
    timer1time: f32,
    timer1time_default: f32,
    message_field_2: MessageCode,
    animation_flag: bool,
    notify_timer: i32,
    timer: i32,
    player_data: [TLightGroupPlayerBackup; 4],
}

impl TLightGroup {
    pub(crate) fn message(&self, code: MessageCode, value: f32) {
        todo!()
    }
}
