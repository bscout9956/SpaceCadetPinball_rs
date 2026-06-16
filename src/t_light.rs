pub struct TlightPlayerBackup {
    message_field: i32,
    light_on_flag: bool,
    light_on_bmp_index: i32,
    flasher_on_flag: bool,
}

pub struct TLight {
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
