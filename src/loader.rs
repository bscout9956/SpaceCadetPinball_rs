use crate::gdrv::GdrvBitmap8;
use crate::group_data::{DatFile, FieldTypes};
use crate::maths::*;
use crate::zdrv::ZMapHeaderType;
use crate::{pb, sound};
use num_traits::Float;
use sdl2::libc::{fclose, fopen, fread};
use sdl2::sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_ERROR;
use sdl2::sys::mixer::Mix_Chunk;
use std::ffi::{CString, c_char, c_void};
use std::ptr::null;
use std::str::FromStr;
use std::sync::atomic::Ordering::Relaxed;

pub struct ErrorMessage {
    code: i32,
    message: &'static str,
}

impl ErrorMessage {
    pub fn get_loader_errors() -> [Self; 28] {
        let error_list = [
            ErrorMessage {
                code: 0,
                message: "Bad Handle",
            },
            ErrorMessage {
                code: 1,
                message: "No Type Field",
            },
            ErrorMessage {
                code: 2,
                message: "No Attributes Field",
            },
            ErrorMessage {
                code: 3,
                message: "Wrong Type: MATERIAL Expected",
            },
            ErrorMessage {
                code: 4,
                message: "Wrong Type: KICKER Expected",
            },
            ErrorMessage {
                code: 5,
                message: "Wrong Type: AN_OBJECT Expected",
            },
            ErrorMessage {
                code: 6,
                message: "Wrong Type: A_STATE Expected",
            },
            ErrorMessage {
                code: 7,
                message: "STATES (re)defined in a state",
            },
            ErrorMessage {
                code: 9,
                message: "Unrecognized Attribute",
            },
            ErrorMessage {
                code: 0x0A,
                message: "Unrecognized float Attribute",
            },
            ErrorMessage {
                code: 0x0B,
                message: "No float Attributes Field",
            },
            ErrorMessage {
                code: 0x0D,
                message: "float Attribute not found",
            },
            ErrorMessage {
                code: 0x0C,
                message: "state_index out of range",
            },
            ErrorMessage {
                code: 0x0F,
                message: "loader_material() reports failure",
            },
            ErrorMessage {
                code: 0x0E,
                message: "loader_kicker() reports failure",
            },
            ErrorMessage {
                code: 0x10,
                message: "loader_state_id() reports failure",
            },
            ErrorMessage {
                code: 0x8,
                message: "# walls doesn't match data size",
            },
            ErrorMessage {
                code: 0x11,
                message: "loader_query_visual_states()",
            },
            ErrorMessage {
                code: 0x12,
                message: "loader_query_visual()",
            },
            ErrorMessage {
                code: 0x15,
                message: "loader_material()",
            },
            ErrorMessage {
                code: 0x14,
                message: "loader_kicker()",
            },
            ErrorMessage {
                code: 0x16,
                message: "loader_query_attribute()",
            },
            ErrorMessage {
                code: 0x17,
                message: "loader_query_iattribute()",
            },
            ErrorMessage {
                code: 0x13,
                message: "loader_query_name()",
            },
            ErrorMessage {
                code: 0x18,
                message: "loader_state_id()",
            },
            ErrorMessage {
                code: 0x19,
                message: "loader_get_sound_id()",
            },
            ErrorMessage {
                code: 0x1A,
                message: "sound reference is not A_SOUND record",
            },
            ErrorMessage {
                code: -1,
                message: "Unknown",
            },
        ];

        error_list
    }
}

pub struct SoundListStruct {
    wave_ptr: *const Mix_Chunk, // TODO: Fixme, I'm a Mix_Chunk
    group_index: i32,
    loaded: bool,
    duration: f32,
}

pub struct VisualKickerStruct {
    pub threshold: f32,
    pub boost: f32,
    pub throw_ball_mult: f32,
    pub throw_ball_direction: Vector3,
    pub throw_ball_angle_mult: f32,
    pub hard_hit_sound_id: i32,
}

pub struct SpriteData {
    bmp: Option<GdrvBitmap8>,
    zmap: Option<ZMapHeaderType>,
}

pub struct VisualStruct {
    pub smoothness: f32,
    pub elasticity: f32,
    pub float_arr_count: i32,
    pub float_arr: &'static [f32], //TODO: should it be static, wtf do I do here?
    pub soft_hit_sound_id: i32,
    pub kicker: VisualKickerStruct,
    pub collision_group: i32,
    pub sound_index_4: i32,
    pub sound_index_3: i32,
    pub bitmap: SpriteData,
}

#[repr(C, packed)]
pub struct WaveHeader {
    riff: [u8; 4],
    overall_size: u32,
    wave: [u8; 4],
    fmt_chunk_marker: [u8; 4],
    length_of_fmt: u32,
    format_type: u16,
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
    data_chunk_header: [u8; 4],
    data_size: u32,
}

// Don't call.
fn compile_time_checks() {
    const { assert!(size_of::<WaveHeader>() == 44) };
}

// TODO: Lifetime check
pub struct Loader<'a> {
    sound_count: i32,
    loader_sound_count: i32,
    loader_table: DatFile<'a>,
    sound_record_table: DatFile<'a>,
    sound_list: [SoundListStruct; 65],
    loader_errors: [ErrorMessage; 28],
}
// TODO: Lifetime check
impl<'a> Loader<'a> {
    // TODO Implement me
    pub fn new() -> Self {
        Self {
            sound_count: 1,
            loader_sound_count: 0,
            loader_table: DatFile {
                app_name: todo!(),
                description: todo!(),
                groups: todo!(),
            },
            sound_record_table: todo!(),
            sound_list: todo!(),
            loader_errors: ErrorMessage::get_loader_errors(),
        }
    }

    pub fn error(&self, error_code: i32, caption_code: i32) -> i32 {
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
        pb::ShowMessageBox(SDL_MESSAGEBOX_ERROR, error_caption, error_text);
        -1
    }

    pub fn default_vsi(mut visual: VisualStruct) {
        visual.collision_group = 0;
        visual.kicker.threshold = 8.99999999;
        visual.kicker.hard_hit_sound_id = 0;
        visual.smoothness = 0.94999999;
        visual.elasticity = 0.60000002;
        visual.float_arr_count = 0;
        visual.soft_hit_sound_id = 0;
        visual.bitmap = SpriteData {
            bmp: None,
            zmap: None,
        };
        visual.sound_index_3 = 0;
        visual.sound_index_3 = 0;
    }

    pub fn load_from(&mut self, dat_file: &DatFile) {
        let loader_table = dat_file;
        let sound_record_table = loader_table;

        for group_index in 0..dat_file.groups.len() as i32 {
            let value = dat_file.field(group_index, FieldTypes::ShortValue);
            if let Some(value_data) = value {
                if (*value_data)[0] == 202 {
                    if self.sound_count < 65 {
                        self.sound_list[self.sound_count as usize] = SoundListStruct {
                            wave_ptr: null(),
                            group_index,
                            loaded: false,
                            duration: 0.0,
                        };
                        self.sound_count += 1;
                    }
                }
            }
        }
        self.loader_sound_count = self.sound_count;
    }

    pub fn unload(&mut self) {
        for index in 1..self.loader_sound_count {
            sound::freesound(self.sound_list[index as usize].wave_ptr);
            // VERIFY: This used to be an empty init, so uh
            // , I guess we can just set everything to null and shit?
            self.sound_list[index as usize] = SoundListStruct {
                wave_ptr: null(),
                group_index: 0,
                loaded: false,
                duration: 0.0,
            }
        }
    }

    pub fn get_sound_id(&mut self, group_index: i32) -> i32 {
        let mut sound_index: i16 = 1;
        if self.sound_count <= 1 {
            self.error(25, 26);
            return -1;
        }

        while (self.sound_list[sound_index as usize].group_index != group_index) {
            sound_index += 1;
            if sound_index as i32 >= self.sound_count {
                self.error(25, 26);
                return -1;
            }
        }

        if (!self.sound_list[sound_index as usize].loaded
            && !self.sound_list[sound_index as usize].wave_ptr.is_null())
        {
            let wave_header: *mut c_void;

            let sound_group_id = self.sound_list[sound_index as usize].group_index;
            self.sound_list[sound_index as usize].duration = 0.0;

            let quick_flag_val = pb::QUICK_FLAG.load(Relaxed);
            if sound_group_id != 0 && !quick_flag_val {
                let value = self
                    .loader_table
                    .field(sound_group_id, FieldTypes::ShortValue);

                if let Some(value_data) = value {
                    if (*value_data)[0] == 202 {
                        // File name is in lower case, while game data is usually in upper case.
                        let file_name_ptr =
                            self.loader_table.field(sound_group_id, FieldTypes::String);
                        if let Some(file_name_data) = file_name_ptr {
                            let mut file_name = String::from_utf8_lossy(file_name_data)
                                .trim_end_matches('\0')
                                .to_string();

                            if pb::FULL_TILT_MODE.load(Relaxed) {
                                // TODO: PathSeparator
                                file_name.insert_str(0, &format!("{}sound", PathSeparator));
                            }

                            let mut file_path = String::new();
                            let mut duration: f32 = -1.0;
                            for idx in 0..3 {
                                if idx == 1 {
                                    file_name.to_uppercase();
                                }

                                file_path = pb::make_path_name(&file_name);
                                let file = unsafe {
                                    fopen(
                                        CString::from_str(&file_path)
                                            .unwrap_or_default()
                                            .into_raw(),
                                        c"rb".as_ptr(),
                                    )
                                };

                                if !file.is_null() {
                                    unsafe {
                                        fread(wave_header, 1, size_of::<WaveHeader>(), file);
                                        fclose(file);
                                        let wave_ptr = &raw mut wave_header as *mut WaveHeader;
                                        let sample_count = ((*wave_ptr).data_size
                                            / (*wave_ptr).channels as u32
                                            * (*wave_ptr).bits_per_sample as u32)
                                            as f64
                                            / 8.0;
                                        duration =
                                            sample_count as f32 / (*wave_ptr).sample_rate as f32;
                                    }
                                }
                            }
                            self.sound_list[sound_index as usize].duration = duration;
                            self.sound_list[sound_index as usize].wave_ptr =
                                sound::load_wave_file(file_path);
                        }
                    }
                }
            }
        }

        self.sound_list[sound_index as usize].loaded = true;
        sound_index as i32
    }

    pub fn query_handle(&self, lp_string: LPCSTR) {
        self.loader_table.record_labeled(lp_string)
    }

    pub fn query_visual_states(&self, group_index: i32) -> i16 {
        let mut result: i16 = 0;

        if group_index < 0 {
            return self.error(0, 17) as i16;
        }

        match self.loader_table.field(group_index, FieldTypes::ShortArray) {
            Some(short_array_data) => {
                if short_array_data.len() >= 4 {
                    let short_value =
                        i16::from_le_bytes([short_array_data[0], short_array_data[1]]);
                    if short_value == 100 {
                        return i16::from_le_bytes([short_array_data[2], short_array_data[3]]);
                    } else {
                        1
                    }
                } else {
                    1
                }
            }
            None => return 1,
        }
    }

    pub fn query_name(&self, group_index: i32) -> *const c_char {
        if group_index < 0 {
            self.error(0, 19);
            return null();
        }

        let result_opt = self.loader_table.field(group_index, FieldTypes::GroupName);
        if let Some(result_data) = result_opt {
            String::from_utf8_lossy(result_data)
                .into_owned()
                .trim_end_matches('\0')
                .to_string()
                .as_ptr() as *const c_char
        } else {
            null()
        }
    }

    pub fn query_iattribute(
        &self,
        group_index: i32,
        first_value: i32,
        array_size: *mut i32,
    ) -> *const i16 {
        if group_index < 0 {
            self.error(0, 20);
            return null::<i16>();
        }

        for skip_index in 0.. {
            match self
                .loader_table
                .field_nth(group_index, FieldTypes::ShortArray, skip_index)
            {
                Some(short_array_data) => {
                    if short_array_data.len() < 2 {
                        continue;
                    }

                    let short_value =
                        i16::from_le_bytes([short_array_data[0], short_array_data[1]]);

                    if short_value == first_value as i16 {
                        unsafe {
                            *array_size = self
                                .loader_table
                                .field_size(group_index, FieldTypes::ShortArray)
                                / 2
                                - 1;
                            return (short_array_data.as_ptr() as *const i16).add(1);
                        }
                    }
                }
                None => {
                    break;
                }
            }
        }

        self.error(2, 23);
        unsafe {
            *array_size = 0;
        }
        null()
    }

    pub fn query_float_attribute_ptr(
        &self,
        group_index: i32,
        group_index_offset: i32,
        first_value: i32,
    ) -> *const f32 {
        if group_index < 0 {
            self.error(0, 22);
            return null::<f32>();
        }

        let state_id = self.state_id(group_index, group_index_offset);
        if state_id < 0 {
            self.error(16, 22);
            return null::<f32>();
        }

        for skip_index in 0..i32::MAX {
            match self
                .loader_table
                .field_nth(group_index, FieldTypes::FloatArray, skip_index)
            {
                Some(float_array_data) => {
                    if float_array_data.len() < 8 {
                        continue;
                    }

                    let float_val = f32::from_le_bytes([
                        float_array_data[0],
                        float_array_data[1],
                        float_array_data[2],
                        float_array_data[3],
                    ]);
                    if (float_val.floor() as i16) == (first_value as i16) {
                        let float_ptr = float_array_data.as_ptr() as *const f32;
                        unsafe { return float_ptr.add(1) }
                    }
                }
                None => {
                    break;
                }
            }
        }

        self.error(13, 22);
        null::<f32>()
    }

    fn query_float_attribute(
        &self,
        group_index: i32,
        group_index_offset: i32,
        first_value: i32,
        def_val: f32,
    ) -> f32 {
        if group_index < 0 {
            self.error(0, 22);
            return f32::nan();
        }

        let state_id = self.state_id(group_index, group_index_offset);
        if state_id < 0 {
            self.error(16, 22);
            return f32::nan();
        }

        for skip_index in 0.. {
            match self
                .loader_table
                .field_nth(group_index, FieldTypes::FloatArray, skip_index)
            {
                Some(float_array_data) => {
                    if float_array_data.len() < 8 {
                        continue;
                    }

                    let float_value = f32::from_le_bytes([
                        float_array_data[0],
                        float_array_data[1],
                        float_array_data[2],
                        float_array_data[3],
                    ]);

                    if (float_value.floor() as i16) == (first_value as i16) {
                        return f32::from_le_bytes([
                            float_array_data[4],
                            float_array_data[5],
                            float_array_data[6],
                            float_array_data[7],
                        ]);
                    }
                }
                None => {
                    break;
                }
            }
        }

        if !def_val.is_nan() {
            return def_val;
        }
        self.error(13, 22);
        f32::nan()
    }

    pub fn material(&self, group_index: i32, visual: *const VisualStruct) {
        if group_index < 0 {
            self.error(0, 21);
        }
        let short_array = self.loader_table.field(group_index, FieldTypes::ShortArray);
    }

    fn state_id(&self, mut group_index: i32, group_index_offset: i32) -> i32 {
        let visual_state = self.query_visual_states(group_index);

        if visual_state <= 0 {
            return self.error(12, 24);
        }
        let mut short_array = self.loader_table.field(group_index, FieldTypes::ShortArray);
        if let Some(short_array_data) = short_array {
            if short_array_data[0] != 200 {
                return self.error(5, 24);
            }
            if group_index_offset > visual_state as i32 {
                return self.error(12, 24);
            }
            if group_index_offset != 0 {
                return group_index;
            }

            group_index += group_index_offset;
        } else {
            return self.error(1, 24);
        }
        short_array = self.loader_table.field(group_index, FieldTypes::ShortValue);
        if let Some(short_array_data) = short_array {
            if short_array_data[0] != 201 {
                return self.error(6, 24);
            }
        } else {
            return self.error(1, 24);
        }

        group_index
    }
}
