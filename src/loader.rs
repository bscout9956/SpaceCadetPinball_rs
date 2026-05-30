use crate::gdrv::GdrvBitmap8;
use crate::group_data::{DatFile, FieldTypes};
use crate::maths::*;
use crate::t_pinball_component::TPinballComponent;
use crate::utils::PATH_SEPARATOR;
use crate::zdrv::ZMapHeaderType;
use crate::{loader, pb, sound};
use num_traits::Float;
use sdl2::libc::{fclose, fopen, fread};
use sdl2::sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_ERROR;
use sdl2::sys::mixer::Mix_Chunk;
use std::ffi::{CString, c_char, c_void};
use std::fs::File;
use std::io::Read;
use std::ptr::null;
use std::str::FromStr;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};

#[derive(Copy, Clone)]
pub struct ErrorMessage {
    code: i32,
    message: &'static str,
}

pub static LOADER_ERRORS: [ErrorMessage; 28] = [
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

#[derive(Copy, Clone)]
pub struct SoundListStruct {
    wave_ptr: *const Mix_Chunk,
    group_index: i32,
    loaded: bool,
    duration: f32,
}

impl Default for SoundListStruct {
    fn default() -> Self {
        Self {
            wave_ptr: null(),
            group_index: 0,
            loaded: false,
            duration: 0.0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct VisualKickerStruct {
    pub threshold: f32,
    pub boost: f32,
    pub throw_ball_mult: f32,
    pub throw_ball_direction: Vector3,
    pub throw_ball_angle_mult: f32,
    pub hard_hit_sound_id: i32,
}

#[derive(Copy, Clone)]
pub struct SpriteData<'a> {
    bmp: Option<GdrvBitmap8<'a>>,
    zmap: Option<ZMapHeaderType>,
}

#[derive(Copy, Clone)]
pub struct VisualStruct<'a> {
    pub smoothness: f32,
    pub elasticity: f32,
    pub float_arr_count: i32,
    pub float_arr: &'a [f32],
    pub soft_hit_sound_id: i32,
    pub kicker: VisualKickerStruct,
    pub collision_group: i32,
    pub sound_index_4: i32,
    pub sound_index_3: i32,
    pub bitmap: SpriteData<'a>,
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

impl Default for WaveHeader {
    fn default() -> Self {
        Self {
            riff: [0; 4],
            overall_size: 0,
            wave: [0; 4],
            fmt_chunk_marker: [0; 4],
            length_of_fmt: 0,
            format_type: 0,
            channels: 0,
            sample_rate: 0,
            byte_rate: 0,
            block_align: 0,
            bits_per_sample: 0,
            data_chunk_header: [0; 4],
            data_size: 0,
        }
    }
}

const _: () = assert!(size_of::<WaveHeader>() == 44, "Wrong size for WaveHeader");

pub struct Loader<'a> {
    sound_count: i32,
    loader_sound_count: i32,
    loader_table: DatFile<'a>,
    sound_record_table: Option<DatFile<'a>>,
    sound_list: [SoundListStruct; 65],
    loader_errors: [ErrorMessage; 28],
}

impl<'a> Loader<'a> {
    pub fn new() -> Self {
        Self {
            sound_count: 1,
            loader_sound_count: 0,
            loader_table: DatFile {
                app_name: "".to_string(),
                description: "".to_string(),
                groups: vec![],
            },
            sound_record_table: None,
            sound_list: [SoundListStruct {
                wave_ptr: null(),
                group_index: 0,
                loaded: false,
                duration: 0.0,
            }; 65],
            loader_errors: LOADER_ERRORS,
        }
    }

    pub fn error(&self, error_code: i32, caption_code: i32) -> i32 {
        let error_text = self
            .loader_errors
            .iter()
            .find(|e| e.code == error_code)
            .map(|e| e.message)
            .unwrap_or("Unknown Error");

        let error_caption = self
            .loader_errors
            .iter()
            .find(|e| e.code == caption_code)
            .map(|e| e.message)
            .unwrap_or("Unknown Error");

        pb::ShowMessageBox(SDL_MESSAGEBOX_ERROR, error_caption, error_text);
        -1
    }

    pub fn default_vsi(visual: &mut VisualStruct<'a>) {
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
        visual.sound_index_4 = 0;
    }

    pub fn load_from(&mut self, dat_file: &DatFile) {
        let loader_table = dat_file;
        let sound_record_table = loader_table;

        for group_index in 0..dat_file.groups.len() as i32 {
            if let Some(value_data) = dat_file.field(group_index, FieldTypes::ShortValue) {
                let final_val = i16::from_le_bytes([(*value_data)[0], (*value_data)[1]]);
                if final_val == 202 {
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
            self.sound_list[index as usize] = SoundListStruct::default();
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
            let mut wave_header = WaveHeader::default();

            let sound_group_id = self.sound_list[sound_index as usize].group_index;
            self.sound_list[sound_index as usize].duration = 0.0;

            let quick_flag_val = pb::QUICK_FLAG.load(Relaxed);
            if sound_group_id != 0 && !quick_flag_val {
                if let Some(value_data) = self
                    .loader_table
                    .field(sound_group_id, FieldTypes::ShortValue)
                {
                    let val = i16::from_le_bytes([value_data[0], value_data[1]]);
                    if val == 202 {
                        // File name is in lower case, while game data is usually in upper case.
                        let file_name_ptr =
                            self.loader_table.field(sound_group_id, FieldTypes::String);
                        if let Some(file_name_data) = file_name_ptr {
                            let mut file_name = String::from_utf8_lossy(file_name_data)
                                .trim_end_matches('\0')
                                .to_string();

                            if pb::FULL_TILT_MODE.load(Relaxed) {
                                file_name.insert_str(0, &format!("{}sound", PATH_SEPARATOR));
                            }

                            let mut file_path = String::new();
                            let mut duration: f32 = -1.0;
                            for idx in 0..2 {
                                if idx == 1 {
                                    file_name = file_name.to_uppercase();
                                }

                                file_path = pb::make_path_name(&file_name);

                                if let Ok(mut file) = File::open(&file_path) {
                                    let mut header_bytes = [0u8; size_of::<WaveHeader>()];
                                    if file.read_exact(&mut header_bytes).is_ok() {
                                        let wave_ptr = unsafe {
                                            &*(header_bytes.as_ptr() as *const WaveHeader)
                                        };
                                        let sample_count = (wave_ptr.data_size
                                            / wave_ptr.channels as u32
                                            * wave_ptr.bits_per_sample as u32)
                                            as f64
                                            / 8.0;
                                        duration =
                                            sample_count as f32 / wave_ptr.sample_rate as f32;
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

    pub fn query_handle(&self, lp_string: *const c_char) -> i32 {
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
                        i16::from_le_bytes([short_array_data[2], short_array_data[3]])
                    } else {
                        1
                    }
                } else {
                    1
                }
            }
            None => 1,
        }
    }

    pub fn query_name(&self, group_index: i32) -> *const c_char {
        if group_index < 0 {
            self.error(0, 19);
            return null();
        }

        if let Some(result_data) = self.loader_table.field(group_index, FieldTypes::GroupName) {
            result_data.as_ptr() as *const c_char
        } else {
            null()
        }
    }

    pub fn query_int_attribute(
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

    pub fn material(&mut self, group_index: i32, visual: *mut VisualStruct) -> i32 {
        if group_index < 0 {
            self.error(0, 21);
        }

        let short_value = match self.loader_table.field(group_index, FieldTypes::ShortValue) {
            Some(short_array_data) => {
                assert_eq!(short_array_data.len(), 2, "Array isn't big enough");
                i16::from_le_bytes([short_array_data[0], short_array_data[1]])
            }
            None => {
                return self.error(1, 21);
            }
        };

        if short_value != 300 {
            return self.error(3, 21);
        }

        let float_array_data = match self.loader_table.field(group_index, FieldTypes::FloatArray) {
            Some(data) => data,
            None => return self.error(11, 21),
        };

        let float_array_len = self
            .loader_table
            .field_size(group_index, FieldTypes::FloatArray)
            / 4;

        for index in (0..float_array_len).step_by(2) {
            let byte_offset = (index * 4) as usize;

            let key = match float_array_data.get(byte_offset..byte_offset + 4) {
                Some(bytes) => f32::from_le_bytes(bytes.try_into().unwrap()),
                None => return self.error(9, 21),
            };

            let value = match float_array_data.get(byte_offset + 4..byte_offset + 8) {
                Some(bytes) => f32::from_le_bytes(bytes.try_into().unwrap()),
                None => return self.error(9, 21),
            };

            match key.floor() as i32 {
                301 => unsafe { (*visual).smoothness = value },
                302 => unsafe { (*visual).elasticity = value },
                304 => unsafe {
                    let sound_id = self.get_sound_id(value.floor() as i32);
                    unsafe { (*visual).soft_hit_sound_id = sound_id }
                },
                _ => return self.error(9, 21),
            }
        }

        0
    }

    pub fn play_sound(
        &self,
        sound_index: i32,
        sound_source: TPinballComponent,
        info: &[u8],
    ) -> f32 {
        if sound_index < 0 {
            return 0.0;
        }

        sound::play_sound(
            self.sound_list[sound_index as usize].wave_ptr,
            pb::TIME_TICKS.load(SeqCst),
            sound_source,
            info,
        );
        self.sound_list[sound_index as usize].duration
    }

    fn state_id(&self, mut group_index: i32, group_index_offset: i32) -> i32 {
        let visual_state = self.query_visual_states(group_index);

        if visual_state <= 0 {
            return self.error(12, 24);
        }
        let mut short_val = match self.loader_table.field(group_index, FieldTypes::ShortValue) {
            Some(data) if data.len() >= 2 => i16::from_le_bytes([data[0], data[1]]),
            _ => return self.error(1, 24),
        };

        if short_val != 200 {
            return self.error(5, 24);
        }
        if group_index_offset > visual_state as i32 {
            return self.error(12, 24);
        }
        if group_index_offset == 0 {
            return group_index;
        }
        group_index += group_index_offset;

        short_val = match self.loader_table.field(group_index, FieldTypes::ShortValue) {
            Some(data) if data.len() >= 2 => i16::from_le_bytes([data[0], data[1]]),
            _ => return self.error(1, 24),
        };

        if short_val != 201 {
            return self.error(6, 24);
        }

        group_index
    }

    fn read_float(&self, data: &[u8], index: &mut usize) -> Result<f32, ()> {
        if *index + 4 > data.len() {
            return Err(());
        }
        let bytes = data[*index..*index + 4].try_into().unwrap();
        *index += 4;
        Ok(f32::from_le_bytes(bytes))
    }

    pub fn kicker(&mut self, group_index: i32, kicker: *mut VisualKickerStruct) -> i32 {
        if group_index < 0 {
            self.error(0, 20);
        }

        let short_value = match self.loader_table.field(group_index, FieldTypes::ShortValue) {
            Some(short_array_data) => {
                assert_eq!(short_array_data.len(), 2, "Array isn't big enough");
                i16::from_le_bytes([short_array_data[0], short_array_data[1]])
            }
            None => {
                return self.error(1, 20);
            }
        };

        if short_value != 400 {
            return self.error(4, 20);
        }

        let float_array_data = match self.loader_table.field(group_index, FieldTypes::FloatArray) {
            Some(data) => data,
            None => return self.error(11, 20),
        };

        let float_array_len = self
            .loader_table
            .field_size(group_index, FieldTypes::FloatArray) as usize;

        let mut index: usize = 0;
        while index < float_array_len {
            let id = match float_array_data.get(index..index + 4) {
                Some(bytes) => f32::from_le_bytes(bytes.try_into().unwrap()) as i32,
                None => return self.error(10, 20),
            };
            index += 4;

            match id {
                401 => unsafe {
                    let val = self.read_float(&float_array_data, &mut index).unwrap();
                    (*kicker).threshold = val
                },
                402 => unsafe {
                    let val = self.read_float(&float_array_data, &mut index).unwrap();
                    (*kicker).boost = val
                },
                403 => unsafe {
                    let val = self.read_float(&float_array_data, &mut index).unwrap();
                    (*kicker).throw_ball_mult = val
                },
                404 => unsafe {
                    let x = self.read_float(&float_array_data, &mut index).unwrap();
                    let y = self.read_float(&float_array_data, &mut index).unwrap();
                    let z = self.read_float(&float_array_data, &mut index).unwrap();
                    (*kicker).throw_ball_direction = Vector3::new(x, y, z);
                },
                405 => unsafe {
                    let val = self.read_float(&float_array_data, &mut index).unwrap();
                    (*kicker).throw_ball_angle_mult = val
                },
                406 => unsafe {
                    let val = self.read_float(&float_array_data, &mut index).unwrap();
                    (*kicker).hard_hit_sound_id = self.get_sound_id(val.floor() as i32);
                },

                _ => return self.error(10, 20),
            }
        }

        0
    }

    pub fn query_visual(
        &mut self,
        group_index: i32,
        group_index_offset: i32,
        visual: &mut VisualStruct<'a>,
    ) -> i32 {
        Self::default_vsi(visual);
        if group_index < 0 {
            return self.error(0, 18);
        }
        let state_id = self.state_id(group_index, group_index_offset);
        if state_id < 0 {
            return self.error(16, 18);
        }

        let bmp = self.loader_table.get_bitmap(state_id);
        let zmap = self.loader_table.get_zmap(state_id);
        visual.bitmap = SpriteData {
            bmp: Some(bmp),
            zmap: Some(zmap),
        };

        let short_array_data = match self.loader_table.field(group_index, FieldTypes::ShortArray) {
            Some(data) => data.to_vec(),
            None => Vec::new(),
        };
        if !short_array_data.is_empty() {
            let short_arr_size = short_array_data.len();
            let mut i: usize = 0;

            while i < short_arr_size {
                if i + 1 >= short_arr_size {
                    break;
                }
                let id = i16::from_le_bytes([short_array_data[i], short_array_data[i + 1]]);
                i += 2;

                match id {
                    100 => {
                        if group_index_offset == 0 {
                            return self.error(7, 18);
                        }
                    }
                    300 => {
                        if i + 1 >= short_arr_size {
                            return self.error(15, 18);
                        }
                        let material_value =
                            i16::from_le_bytes([short_array_data[i], short_array_data[i + 1]]);
                        i += 2;
                        if self.material(material_value as i32, visual as *mut _) != 0 {
                            return self.error(15, 18);
                        }
                    }
                    304 => {
                        if i + 1 >= short_arr_size {
                            break;
                        }
                        let sound_id =
                            i16::from_le_bytes([short_array_data[i], short_array_data[i + 1]]);
                        i += 2;
                        visual.soft_hit_sound_id = self.get_sound_id(sound_id as i32);
                    }
                    400 => {
                        if i + 1 >= short_arr_size {
                            break;
                        }
                        let kicker_val =
                            i16::from_le_bytes([short_array_data[i], short_array_data[i + 1]]);
                        i += 2;
                        // VERIFY: Is the 0 check correct? Should it be not 0?
                        if self.kicker(kicker_val as i32, &mut visual.kicker) != 0 {
                            return self.error(14, 18);
                        }
                    }
                    406 => {
                        if i + 1 >= short_arr_size {
                            break;
                        }

                        let sound_id =
                            i16::from_le_bytes([short_array_data[i], short_array_data[i + 1]]);
                        i += 2;
                        visual.kicker.hard_hit_sound_id = self.get_sound_id(sound_id as i32);
                    }
                    602 => {
                        if i + 1 >= short_arr_size {
                            break;
                        }
                        let shift =
                            i16::from_le_bytes([short_array_data[i], short_array_data[i + 1]]);
                        i += 2;
                        visual.collision_group |= 1 << shift;
                    }
                    1100 => {
                        if i + 1 >= short_arr_size {
                            break;
                        }
                        let sound_id =
                            i16::from_le_bytes([short_array_data[i], short_array_data[i + 1]]);
                        i += 2;
                        visual.sound_index_4 = self.get_sound_id(sound_id as i32);
                    }
                    1101 => {
                        if i + 1 >= short_arr_size {
                            break;
                        }
                        let sound_id =
                            i16::from_le_bytes([short_array_data[i], short_array_data[i + 1]]);
                        i += 2;
                        visual.sound_index_3 = self.get_sound_id(sound_id as i32);
                    }
                    1500 => {
                        // Skipping 7 shorts or 14 bytes
                        i += 14;
                    }
                    _ => {
                        return self.error(9, 18);
                    }
                }
            }
        }
        // VERIFY: Is the 0 check correct? Should it be not 0?
        if visual.collision_group != 0 {
            visual.collision_group = 1;
        }

        let float_array_data = match self.loader_table.field(group_index, FieldTypes::FloatArray) {
            Some(float_array_data) => float_array_data.to_vec(),
            None => Vec::new(),
        };
        if !float_array_data.is_empty() {
            let float_val = f32::from_le_bytes([
                float_array_data[0],
                float_array_data[1],
                float_array_data[2],
                float_array_data[3],
            ]);

            if float_val != 600.0 {
                return 0;
            }

            visual.float_arr_count = self
                .loader_table
                .field_size(group_index, FieldTypes::FloatArray)
                / 4
                / 2
                - 2;

            let float_int = (f32::from_le_bytes([
                float_array_data[4],
                float_array_data[5],
                float_array_data[6],
                float_array_data[7],
            ])
            .floor()
                - 1.0) as i32;
            match float_int {
                0 => visual.float_arr_count = 1,
                1 => visual.float_arr_count = 2,
                _ => {
                    if float_int != visual.float_arr_count {
                        return self.error(8, 18);
                    }
                }
            }

            let mut arr = Vec::with_capacity(visual.float_arr_count as usize);
            for i in 0..visual.float_arr_count as usize {
                let base = 8 + (i * 4);
                let val = f32::from_le_bytes(float_array_data[base..base + 4].try_into().unwrap());
                arr.push(val);
            }
            visual.float_arr = arr.leak();
        }

        0
    }
}
