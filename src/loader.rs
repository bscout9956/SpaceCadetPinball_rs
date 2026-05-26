use std::ffi::c_void;
use sdl2::sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_ERROR;
use crate::maths::Vector3;

pub struct ErrorMessage {
    code: i32,
    message: &'static str,
}

impl ErrorMessage {
    pub fn get_loader_errors() -> [Self; 28] {
        let error_list = [
            ErrorMessage {code: 0, message: "Bad Handle"},
            ErrorMessage {code: 1, message: "No Type Field"},
            ErrorMessage {code: 2, message: "No Attributes Field"},
            ErrorMessage {code: 3, message: "Wrong Type: MATERIAL Expected"},
            ErrorMessage {code: 4, message: "Wrong Type: KICKER Expected"},
            ErrorMessage {code: 5, message: "Wrong Type: AN_OBJECT Expected"},
            ErrorMessage {code: 6, message: "Wrong Type: A_STATE Expected"},
            ErrorMessage {code: 7, message: "STATES (re)defined in a state"},
            ErrorMessage {code: 9, message: "Unrecognized Attribute"},
            ErrorMessage {code: 0x0A, message: "Unrecognized float Attribute"},
            ErrorMessage {code: 0x0B, message: "No float Attributes Field"},
            ErrorMessage {code: 0x0D, message: "float Attribute not found"},
            ErrorMessage {code: 0x0C, message: "state_index out of range"},
            ErrorMessage {code: 0x0F, message: "loader_material() reports failure"},
            ErrorMessage {code: 0x0E, message: "loader_kicker() reports failure"},
            ErrorMessage {code: 0x10, message: "loader_state_id() reports failure"},
            ErrorMessage {code: 0x8, message: "# walls doesn't match data size"},
            ErrorMessage {code: 0x11, message: "loader_query_visual_states()"},
            ErrorMessage {code: 0x12, message: "loader_query_visual()"},
            ErrorMessage {code: 0x15, message: "loader_material()"},
            ErrorMessage {code: 0x14, message: "loader_kicker()"},
            ErrorMessage {code: 0x16, message: "loader_query_attribute()"},
            ErrorMessage {code: 0x17, message: "loader_query_iattribute()"},
            ErrorMessage {code: 0x13, message: "loader_query_name()"},
            ErrorMessage {code: 0x18, message: "loader_state_id()"},
            ErrorMessage {code: 0x19, message: "loader_get_sound_id()"},
            ErrorMessage {code: 0x1A, message: "sound reference is not A_SOUND record"},
            ErrorMessage {code: -1, message: "Unknown"},
        ];

        error_list
    }
}

pub struct SoundListStruct {
    wave_ptr: c_void, // TODO: Fixme, I'm a Mix_Chunk
    group_index: i32,
    loaded: bool,
    duration: f32,
}

pub struct VisualKickerStruct {
    threshold: f32,
    boost: f32,
    throw_ball_mult: f32,
    throw_ball_direction: Vector3,
    throw_ball_angle_mult: f32,
    hard_hit_sound_id: i32,
}

// TODO: Implement me
struct ZMap {

}

struct gdrv_bitmap8; // TODO: Implement me

pub struct SpriteData {
    bmp: gdrv_bitmap8,
    zmap_header_type: ZMap,
}

pub struct VisualStruct {
    smoothness: f32,
    elasticity: f32,
    float_arr_count: i32,
    float_arr: &'static [f32], //TODO: should it be static, wtf do I do here?
    soft_hit_sound_id: i32,
    kicker: VisualKickerStruct,
    collision_group: i32,
    sound_index_4:i32,
    sound_index_3:i32,
    bitmap: SpriteData
}

#[repr(C, packed)]
pub struct WaveHeader {
    riff: [u8;4],
    overall_size: u32,
    wave: [u8;4],
    fmt_chunk_marker: [u8;4],
    length_of_fmt: u32,
    format_type: u16,
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
    data_chunk_header: [u8;4],
    data_size: u32,
}

// Don't call.
fn compile_time_checks() {
    const {assert!(std::mem::size_of::<WaveHeader>() == 44)};
}

pub struct Loader {
    sound_count: i32, // TODO: Init to 1
    loader_sound_count: i32,
    loader_table: DatFile,
    sound_record_table: DatFile,
    sound_list: [SoundListStruct; 65],
    loader_errors: [ErrorMessage; 28],
}

impl Loader {
    // TODO Implement me
    fn new() -> Self {
        Self {
            sound_count: 0,
            loader_sound_count: 0,
            loader_table: (),
            sound_record_table: (),
            sound_list: [],
            loader_errors: ErrorMessage::get_loader_errors(),
        }
    }

    fn error(&self, error_code: i32, caption_code: i32) -> i32 {
        let mut error_text: &str = "";
        let mut error_caption: &str;

        let mut index = 0;
        for error in &self.loader_errors {
            if error_code == error.code {
                error_text = error.message;
            }
            if caption_code == error.code {
                error_caption = error.message;
            }
            index = index + 1;
        }

        if error_text.is_empty() {
            error_text = self.loader_errors[index].message;
        }
        // TODO: Implement me
        pb::ShowMessageBox(SDL_MESSAGEBOX_ERROR, error_caption, error_text);
        -1
    }

    fn default_vsi(mut visual: VisualStruct) {
        visual.collision_group = 0;
        visual.kicker.threshold = 8.99999999;
        visual.kicker.hard_hit_sound_id = 0;
        visual.smoothness = 0.94999999;
        visual.elasticity = 0.60000002;
        visual.float_arr_count = 0;
        visual.soft_hit_sound_id = 0;
        // TODO: Implement me?
        visual.bitmap = SpriteData::new();
        visual.sound_index_3 = 0;
        visual.sound_index_3 = 0;
    }
}
