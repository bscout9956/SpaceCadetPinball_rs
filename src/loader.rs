use crate::context::component_context::ComponentContext;
use crate::errors::LoaderError;
use crate::gdrv::GdrvBitmap8;
use crate::group_data::{DatFile, EntryBuffer, FieldTypes};
use crate::maths::*;
use crate::state::loader_state::LoaderState;
use crate::state::pb_game_state::PbGameState;
use crate::state::pinball_state::PinballState;
use crate::state::sound_state::SoundState;
use crate::t_pinball_component::IPinballComponent;
use crate::utils::{PATH_SEPARATOR, SdlWindowPtr};
use crate::zdrv::ZMapHeaderType;
use crate::{pb, sound};
use anyhow::bail;
use anyhow::{Context, Result};
use num_traits::Float;
use sdl2::sys::SDL_MessageBoxFlags::SDL_MESSAGEBOX_ERROR;
use sdl2::sys::mixer::Mix_Chunk;
use std::ffi::{CStr, c_char};
use std::fs::File;
use std::io::Read;
use std::ptr::null;
use std::sync::{Arc, RwLock};

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
    pub wave: Option<Mix_Chunk>,
    pub group_index: i32,
    pub loaded: bool,
    pub duration: f32,
}

unsafe impl Sync for SoundListStruct {}
unsafe impl Send for SoundListStruct {}

impl Default for SoundListStruct {
    fn default() -> Self {
        Self {
            wave: None,
            group_index: 0,
            loaded: false,
            duration: 0.0,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct VisualKickerStruct {
    pub threshold: f32,
    pub boost: f32,
    pub throw_ball_mult: f32,
    pub throw_ball_direction: Vector3,
    pub throw_ball_angle_mult: f32,
    pub hard_hit_sound_id: i32,
}

#[derive(Clone, Default)]
pub struct SpriteData {
    pub bmp: Arc<Option<GdrvBitmap8>>,
    pub zmap: Arc<Option<ZMapHeaderType>>,
}

#[derive(Clone, Default)]
pub struct VisualStruct {
    pub smoothness: f32,
    pub elasticity: f32,
    pub float_arr_count: i32, // WARNING! If you use this in a loop, multiply by two, or rather use float_arr.len()
    pub float_arr: Vec<f32>,
    pub soft_hit_sound_id: i32,
    pub kicker: VisualKickerStruct,
    pub collision_group: i32,
    pub sound_index_4: i32,
    pub sound_index_3: i32,
    pub bitmap: SpriteData,
}

#[repr(C, packed)]
#[derive(Default)]
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

const _: () = assert!(size_of::<WaveHeader>() == 44, "Wrong size for WaveHeader");

pub fn error(error_code: i32, caption_code: i32) -> Result<i32> {
    let error_text = LOADER_ERRORS
        .iter()
        .find(|e| e.code == error_code)
        .map(|e| e.message)
        .unwrap_or("Unknown Error");

    let error_caption = LOADER_ERRORS
        .iter()
        .find(|e| e.code == caption_code)
        .map(|e| e.message)
        .unwrap_or("Unknown Error");

    // HACK: I am passing no window here because this would mean error
    // would need to know the window resulting in multiple calls being affected by it
    let none_window: &Option<SdlWindowPtr> = &Option::None;
    pb::show_message_box(SDL_MESSAGEBOX_ERROR, error_caption, error_text, none_window)
        .context("Failed to display MessageBox for error")?;
    Ok(-1)
}

pub fn default_vsi(visual: &mut VisualStruct) {
    visual.collision_group = 0;
    visual.kicker.threshold = 8.999_999_99;
    visual.kicker.hard_hit_sound_id = 0;
    visual.smoothness = 0.95;
    visual.elasticity = 0.6;
    visual.float_arr_count = 0;
    visual.soft_hit_sound_id = 0;
    visual.bitmap = SpriteData {
        bmp: Arc::new(None),
        zmap: Arc::new(None),
    };
    visual.sound_index_3 = 0;
    visual.sound_index_4 = 0;
}

pub fn load_from(
    dat: Arc<RwLock<DatFile>>,
    loader_state: &mut LoaderState,
) -> Result<(), LoaderError> {
    loader_state.loader_table = Some(Arc::clone(&dat));
    loader_state.sound_record_table = Some(Arc::clone(&dat));

    let shared_dat = dat.read().unwrap();

    let groups_len = shared_dat.groups.len() as i32;
    for group_index in 0..groups_len {
        if let Some(EntryBuffer::Raw(value_data)) =
            shared_dat.field(group_index, FieldTypes::ShortValue)
        {
            let final_val = i16::from_le_bytes([(*value_data)[0], (*value_data)[1]]);
            if final_val == 202 && loader_state.sound_count < 65 {
                let sound_count = loader_state.sound_count as usize;
                loader_state.sound_list[sound_count] = SoundListStruct {
                    wave: None,
                    group_index,
                    loaded: false,
                    duration: 0.0,
                };
                loader_state.sound_count += 1;
            }
        }
    }

    loader_state.loader_sound_count = loader_state.sound_count;
    Ok(())
}

pub fn unload(
    loader_state: &mut LoaderState,
    sound_state: &mut SoundState,
) -> Result<(), LoaderError> {
    for index in 1..loader_state.loader_sound_count {
        sound::free_sound(loader_state.sound_list[index as usize].wave, sound_state);
        loader_state.sound_list[index as usize] = SoundListStruct::default();
    }
    Ok(())
}

pub fn get_sound_id(
    group_index: i32,
    pb_game_state: &mut PbGameState,
    loader_state: &mut LoaderState,
    sound_state: &mut SoundState,
) -> Result<i32> {
    let mut sound_index: i16 = 1;
    if loader_state.sound_count <= 1 {
        error(25, 26)?;
        return Ok(-1);
    }

    while loader_state.sound_list[sound_index as usize].group_index != group_index {
        sound_index += 1;
        if sound_index as i32 >= loader_state.sound_count {
            error(25, 26)?;
            return Ok(-1);
        }
    }

    if !loader_state.sound_list[sound_index as usize].loaded
        && loader_state.sound_list[sound_index as usize].wave.is_none()
    {
        let sound_group_id = loader_state.sound_list[sound_index as usize].group_index;
        loader_state.sound_list[sound_index as usize].duration = 0.0;

        let table_arc = loader_state.loader_table.as_ref().unwrap();
        let loader_table = table_arc.read().map_err(|_| LoaderError::TableLock)?;
        if sound_group_id != 0
            && !pb_game_state.quick_flag
            && let Some(EntryBuffer::Raw(value_data)) =
                loader_table.field(sound_group_id, FieldTypes::ShortValue)
        {
            let val = i16::from_le_bytes([value_data[0], value_data[1]]);
            if val == 202 {
                // File name is in lower case, while game data is usually in upper case.

                let file_name_ptr = loader_table.field(sound_group_id, FieldTypes::String);
                if let Some(EntryBuffer::Raw(file_name_data)) = file_name_ptr {
                    let mut file_name = String::from_utf8_lossy(file_name_data.as_ref())
                        .trim_end_matches('\0')
                        .to_string();

                    if pb_game_state.full_tilt_mode {
                        file_name.insert_str(0, &format!("{}sound", PATH_SEPARATOR));
                    }

                    let mut file_path = String::new();
                    let mut duration: f32 = -1.0;
                    for idx in 0..2 {
                        if idx == 1 {
                            file_name = file_name.to_uppercase();
                        }

                        file_path = pb::make_path_name(&file_name, &pb_game_state.base_path);

                        if let Ok(mut file) = File::open(&file_path) {
                            let mut header_bytes = [0u8; size_of::<WaveHeader>()];
                            if file.read_exact(&mut header_bytes).is_ok() {
                                let wave_ptr =
                                    unsafe { &*(header_bytes.as_ptr() as *const WaveHeader) };
                                let sample_count = wave_ptr.data_size as f32
                                    / (wave_ptr.channels as f32
                                        * (wave_ptr.bits_per_sample as f32 / 8.0));
                                duration = sample_count / wave_ptr.sample_rate as f32;
                                break;
                            }
                        }
                    }
                    loader_state.sound_list[sound_index as usize].duration = duration;
                    loader_state.sound_list[sound_index as usize].wave =
                        sound::load_wave_file(file_path.as_str(), sound_state)?;
                }
            }
        }
    }

    loader_state.sound_list[sound_index as usize].loaded = true;
    Ok(sound_index as i32)
}

pub fn query_handle(
    lp_string: *const c_char,
    loader_state: &mut LoaderState,
) -> Result<i32, LoaderError> {
    let table_arc = loader_state.loader_table.as_ref().unwrap();
    let loader_table = table_arc.read().unwrap();
    let lp_str = unsafe { CStr::from_ptr(lp_string).to_string_lossy().into_owned() };
    Ok(loader_table.record_labeled(&lp_str))
}

// TODO: Might be able to define new types in the EntryBuffer enum

pub fn query_visual_states(group_index: i32, loader_state: &mut LoaderState) -> Result<i16> {
    if group_index < 0 {
        let err = error(0, 17)?;
        return Ok(err as i16);
    }

    let table_arc = loader_state.loader_table.as_ref().unwrap();
    let loader_table = table_arc.read().unwrap();

    match loader_table.field(group_index, FieldTypes::ShortArray) {
        Some(EntryBuffer::Raw(short_array_data)) if short_array_data.len() >= 4 => {
            let short_value = i16::from_le_bytes([short_array_data[0], short_array_data[1]]);
            if short_value == 100 {
                Ok(i16::from_le_bytes([
                    short_array_data[2],
                    short_array_data[3],
                ]))
            } else {
                Ok(1)
            }
        }
        // Bitmap or none
        _ => Ok(1),
    }
}

pub fn query_name(group_index: i32, loader_state: &mut LoaderState) -> Result<String> {
    let table_arc = loader_state.loader_table.as_ref().unwrap();
    let loader_table = table_arc.read().map_err(|_| LoaderError::TableLock)?;
    if group_index < 0 {
        error(0, 19)?;
        return Ok("".to_string());
    }

    if let Some(EntryBuffer::Raw(result_data)) =
        loader_table.field(group_index, FieldTypes::GroupName)
    {
        let c_str = CStr::from_bytes_with_nul(result_data)?; // Assuming we have a nul terminator there, should be ok?
        Ok(c_str.to_string_lossy().into_owned())
    } else {
        Ok("".to_string())
    }
}

pub fn query_int_attribute(
    group_index: i32,
    first_value: i32,
    array_size: *mut i32,
    loader_state: &mut LoaderState,
) -> Result<*const i16> {
    if group_index < 0 {
        error(0, 20)?;
        return Ok(null::<i16>());
    }

    let table_arc = loader_state
        .loader_table
        .as_ref()
        .context("Failed to obtain loader table as reference")?;
    let loader_table = table_arc.read().map_err(|_| LoaderError::TableLock)?;
    for skip_index in 0.. {
        match loader_table.field_nth(group_index, FieldTypes::ShortArray, skip_index) {
            Some(EntryBuffer::Raw(short_array_data)) => {
                if short_array_data.len() < 2 {
                    continue;
                }

                let short_value = i16::from_le_bytes([short_array_data[0], short_array_data[1]]);

                if short_value == first_value as i16 {
                    unsafe {
                        *array_size =
                            loader_table.field_size(group_index, FieldTypes::ShortArray) / 2 - 1;
                        return Ok((short_array_data.as_ptr() as *const i16).add(1));
                    }
                }
            }
            // Bitmap or None
            _ => {
                break;
            }
        }
    }

    error(2, 23)?;
    unsafe {
        *array_size = 0;
    }
    Ok(null())
}

pub fn query_float_attribute_ptr(
    group_index: i32,
    group_index_offset: i32,
    first_value: i32,
    loader_state: &mut LoaderState,
) -> Result<*const f32> {
    if group_index < 0 {
        error(0, 22)?;
        return Ok(null::<f32>());
    }

    let state_id = state_id(group_index, group_index_offset, loader_state)?;
    if state_id < 0 {
        error(16, 22)?;
        return Ok(null::<f32>());
    }

    let table_arc = loader_state.loader_table.as_ref().unwrap();
    let loader_table = table_arc.read().unwrap();

    for skip_index in 0..i32::MAX {
        match loader_table.field_nth(group_index, FieldTypes::FloatArray, skip_index) {
            Some(EntryBuffer::Raw(float_array_data)) => {
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
                    unsafe {
                        return Ok(float_ptr.add(1));
                    }
                }
            }
            //Bitmap or none
            _ => {
                break;
            }
        }
    }

    error(13, 22)?;
    Ok(null::<f32>())
}
pub fn query_float_attribute(
    group_index: i32,
    group_index_offset: i32,
    first_value: i32,
    def_val: f32,
    loader_state: &mut LoaderState,
) -> Result<f32> {
    if group_index < 0 {
        error(0, 22)?;
        return Ok(f32::nan());
    }

    let state_id = state_id(group_index, group_index_offset, loader_state)?;
    if state_id < 0 {
        error(16, 22)?;
        return Ok(f32::nan());
    }

    let table_arc = loader_state.loader_table.as_ref().unwrap();
    let loader_table = table_arc.read().unwrap();
    for skip_index in 0.. {
        match loader_table.field_nth(group_index, FieldTypes::FloatArray, skip_index) {
            Some(EntryBuffer::Raw(float_array_data)) => {
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
                    return Ok(f32::from_le_bytes([
                        float_array_data[4],
                        float_array_data[5],
                        float_array_data[6],
                        float_array_data[7],
                    ]));
                }
            }
            _ => {
                break;
            }
        }
    }

    if !def_val.is_nan() {
        return Ok(def_val);
    }
    error(13, 22)?;
    Ok(f32::nan())
}

pub fn material(
    group_index: i32,
    visual: *mut VisualStruct,
    pb_game_state: &mut PbGameState,
    loader_state: &mut LoaderState,
    sound_state: &mut SoundState,
) -> Result<i32> {
    if group_index < 0 {
        error(0, 21)?;
    }

    let mut float_array_data = Vec::new();
    let mut float_array_len = 0;
    if let Some(table_arc) = loader_state.loader_table.as_ref() {
        let loader_table = table_arc.read().map_err(|_| LoaderError::TableLock)?;
        let short_value = match loader_table.field(group_index, FieldTypes::ShortValue) {
            Some(EntryBuffer::Raw(short_array_data)) => {
                assert_eq!(short_array_data.len(), 2, "Array isn't big enough");
                i16::from_le_bytes([short_array_data[0], short_array_data[1]])
            }
            _ => {
                return error(1, 21);
            }
        };

        if short_value != 300 {
            return error(3, 21);
        }

        float_array_data = match loader_table.field(group_index, FieldTypes::FloatArray) {
            Some(EntryBuffer::Raw(data)) => data.to_vec(),
            _ => return error(11, 21),
        };

        float_array_len = loader_table.field_size(group_index, FieldTypes::FloatArray) / 4;
    }

    for index in (0..float_array_len).step_by(2) {
        let byte_offset = (index * 4) as usize;

        let key = match float_array_data.get(byte_offset..byte_offset + 4) {
            Some(bytes) => f32::from_le_bytes(bytes.try_into()?),
            None => return error(9, 21),
        };

        let value = match float_array_data.get(byte_offset + 4..byte_offset + 8) {
            Some(bytes) => f32::from_le_bytes(bytes.try_into()?),
            None => return error(9, 21),
        };

        match key.floor() as i32 {
            301 => unsafe { (*visual).smoothness = value },
            302 => unsafe { (*visual).elasticity = value },
            304 => {
                let sound_id = get_sound_id(
                    value.floor() as i32,
                    pb_game_state,
                    loader_state,
                    sound_state,
                )?;
                unsafe { (*visual).soft_hit_sound_id = sound_id }
            }
            _ => return error(9, 21),
        }
    }

    Ok(0)
}

pub fn play_sound(
    sound_index: i32,
    sound_source: Option<&dyn IPinballComponent>,
    info: &str,
    component_context: &mut ComponentContext,
) -> f32 {
    if sound_index <= 0 {
        return 0.0;
    }

    let Some(loader_state) = component_context.loader_state.as_ref() else {
        return 0.0;
    };
    let Some(sound_state) = component_context.sound_state.as_mut() else {
        return 0.0;
    };

    let sound = loader_state.sound_list[sound_index as usize];
    sound::play_sound(
        sound_state,
        sound.wave,
        component_context.time_ticks,
        component_context.sound_stereo,
        component_context.edge_manager,
        sound_source,
        info,
    );
    sound.duration
}

fn state_id(
    mut group_index: i32,
    group_index_offset: i32,
    loader_state: &mut LoaderState,
) -> Result<i32> {
    let visual_state =
        query_visual_states(group_index, loader_state).context("Error querying visual states")?;

    if visual_state <= 0 {
        return error(12, 24);
    }

    let table_arc = loader_state.loader_table.as_ref().unwrap();
    let loader_table = table_arc.read().map_err(|_| LoaderError::TableLock)?;

    let mut short_val = match loader_table.field(group_index, FieldTypes::ShortValue) {
        Some(EntryBuffer::Raw(data)) if data.len() >= 2 => i16::from_le_bytes([data[0], data[1]]),
        _ => return error(1, 24),
    };

    if short_val != 200 {
        return error(5, 24);
    }
    if group_index_offset > visual_state as i32 {
        return error(12, 24);
    }
    if group_index_offset == 0 {
        return Ok(group_index);
    }
    group_index += group_index_offset;

    short_val = match loader_table.field(group_index, FieldTypes::ShortValue) {
        Some(EntryBuffer::Raw(data)) if data.len() >= 2 => i16::from_le_bytes([data[0], data[1]]),
        _ => return error(1, 24),
    };

    if short_val != 201 {
        return error(6, 24);
    }

    Ok(group_index)
}

fn read_float(data: &[u8], index: &mut usize) -> Result<f32> {
    if *index + 4 > data.len() {
        bail!("Index greater than data length");
    }
    let bytes = data[*index..*index + 4].try_into()?;
    *index += 4;
    Ok(f32::from_le_bytes(bytes))
}

pub fn kicker(
    group_index: i32,
    kicker: *mut VisualKickerStruct,
    pb_game_state: &mut PbGameState,
    loader_state: &mut LoaderState,
    sound_state: &mut SoundState,
) -> Result<i32> {
    if group_index < 0 {
        error(0, 20)?;
    }

    if let Some(table_arc) = loader_state.loader_table.as_ref() {
        let loader_table = table_arc.read().map_err(|_| LoaderError::TableLock)?;
        let short_value = match loader_table.field(group_index, FieldTypes::ShortValue) {
            Some(EntryBuffer::Raw(data)) => {
                assert_eq!(data.len(), 2, "Array isn't big enough");
                i16::from_le_bytes([data[0], data[1]])
            }
            _ => {
                return error(1, 20);
            }
        };
        if short_value != 400 {
            return error(4, 20);
        }
    }

    let mut float_array_data = Vec::new();
    let mut float_array_len = 0;

    if let Some(table_arc) = loader_state.loader_table.as_ref() {
        let loader_table = table_arc.read().map_err(|_| LoaderError::TableLock)?;
        float_array_data = match loader_table.field(group_index, FieldTypes::FloatArray) {
            Some(EntryBuffer::Raw(data)) => data.to_vec(),
            _ => return error(11, 20),
        };
        float_array_len = loader_table.field_size(group_index, FieldTypes::FloatArray) as usize;
    }

    let mut index: usize = 0;
    while index < float_array_len {
        let id = match float_array_data.get(index..index + 4) {
            Some(bytes) => f32::from_le_bytes(bytes.try_into()?) as i32,
            None => return error(10, 20),
        };
        index += 4;

        match id {
            401 => unsafe {
                let val = read_float(&float_array_data, &mut index).unwrap();
                (*kicker).threshold = val
            },
            402 => unsafe {
                let val = read_float(&float_array_data, &mut index).unwrap();
                (*kicker).boost = val
            },
            403 => unsafe {
                let val = read_float(&float_array_data, &mut index).unwrap();
                (*kicker).throw_ball_mult = val
            },
            404 => unsafe {
                let x = read_float(&float_array_data, &mut index).unwrap();
                let y = read_float(&float_array_data, &mut index).unwrap();
                let z = read_float(&float_array_data, &mut index).unwrap();
                (*kicker).throw_ball_direction = Vector3::new(x, y, z);
            },
            405 => unsafe {
                let val = read_float(&float_array_data, &mut index).unwrap();
                (*kicker).throw_ball_angle_mult = val
            },
            406 => unsafe {
                let val = read_float(&float_array_data, &mut index)
                    .context("Error reading float for kicker")?;
                (*kicker).hard_hit_sound_id =
                    get_sound_id(val.floor() as i32, pb_game_state, loader_state, sound_state)?;
            },

            _ => return error(10, 20),
        }
    }

    Ok(0)
}

pub fn query_visual(
    group_index: i32,
    group_index_offset: i32,
    visual: &mut VisualStruct,
    state: &mut PinballState,
) -> Result<i32> {
    default_vsi(visual);
    if group_index < 0 {
        return error(0, 18);
    }
    let state_id = state_id(group_index, group_index_offset, &mut state.loader_state)?;
    if state_id < 0 {
        return error(16, 18);
    }

    let mut short_array_data = Vec::new();
    let mut float_array_data = Vec::new();
    let mut float_arr_count = 0;

    if let Some(table_arc) = state.loader_state.loader_table.as_ref() {
        let loader_table = table_arc.read().map_err(|_| LoaderError::TableLock)?;

        let bmp = loader_table
            .get_bitmap(state_id, state.fullscrn_state.resolution)
            .to_owned();
        let zmap = loader_table
            .get_zmap(state_id, state.fullscrn_state.resolution)
            .to_owned();

        visual.bitmap = SpriteData {
            bmp: Arc::new(bmp),
            zmap: Arc::new(zmap),
        };

        short_array_data = match loader_table.field(state_id, FieldTypes::ShortArray) {
            Some(EntryBuffer::Raw(data)) => data.to_vec(),
            _ => vec![],
        };

        float_array_data = match loader_table.field(state_id, FieldTypes::FloatArray) {
            Some(EntryBuffer::Raw(data)) => data.to_vec(),
            _ => vec![],
        };

        float_arr_count = loader_table.field_size(state_id, FieldTypes::FloatArray) / 4 / 2 - 2;
    }

    if !short_array_data.is_empty() {
        let short_arr_size = short_array_data.len();
        let mut i: usize = 0;

        // Loop runs as long as we have at least 4 bytes left, 2 for id then 2 for value
        while i + 3 < short_arr_size {
            if i + 1 >= short_arr_size {
                break;
            }
            let id = i16::from_le_bytes([short_array_data[i], short_array_data[i + 1]]);
            let value = i16::from_le_bytes([short_array_data[i + 2], short_array_data[i + 3]]);

            match id {
                100 => {
                    if group_index_offset > 0 {
                        return error(7, 18);
                    }
                }
                300 => {
                    if material(
                        value as i32,
                        visual as *mut _,
                        &mut state.pb_game_state,
                        &mut state.loader_state,
                        &mut state.sound_state,
                    )? != 0
                    {
                        return error(15, 18);
                    }
                }
                304 => {
                    if i + 1 >= short_arr_size {
                        break;
                    }
                    visual.soft_hit_sound_id = get_sound_id(
                        value as i32,
                        &mut state.pb_game_state,
                        &mut state.loader_state,
                        &mut state.sound_state,
                    )?;
                }
                400 => {
                    if kicker(
                        value as i32,
                        &mut visual.kicker,
                        &mut state.pb_game_state,
                        &mut state.loader_state,
                        &mut state.sound_state,
                    )? != 0
                    {
                        return error(14, 18);
                    }
                }
                406 => {
                    visual.kicker.hard_hit_sound_id = get_sound_id(
                        value as i32,
                        &mut state.pb_game_state,
                        &mut state.loader_state,
                        &mut state.sound_state,
                    )?;
                }
                602 => {
                    visual.collision_group |= 1 << value;
                }
                1100 => {
                    visual.sound_index_4 = get_sound_id(
                        value as i32,
                        &mut state.pb_game_state,
                        &mut state.loader_state,
                        &mut state.sound_state,
                    )?;
                }
                1101 => {
                    visual.sound_index_3 = get_sound_id(
                        value as i32,
                        &mut state.pb_game_state,
                        &mut state.loader_state,
                        &mut state.sound_state,
                    )?;
                }
                1500 => {
                    // C++ does shortArr += 7. Since each element is 2 bytes,
                    // we skip an additional 14 bytes here.
                    i += 14;
                }
                _ => {
                    return error(9, 18);
                }
            }
            // Advance by 4 bytes to reach the next id/value pair
            i += 4
        }
    }

    if visual.collision_group == 0 {
        visual.collision_group = 1;
    }

    if !float_array_data.is_empty() {
        let floats: Vec<f32> = float_array_data
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();

        if floats.is_empty() || floats[0] != 600.0 {
            return Ok(0);
        }

        visual.float_arr_count = float_arr_count;

        let float_pair_count = (floats[1].floor() - 1.0f32) as i32;
        match float_pair_count {
            0 => visual.float_arr_count = 1,
            1 => visual.float_arr_count = 2,
            _ => {
                if float_pair_count != visual.float_arr_count {
                    return error(8, 18);
                }
            }
        }

        let limit = (float_pair_count as usize + 2) * 2; // cnt = 10? limit: 12?
        visual.float_arr = floats[2..limit].to_vec();
    }

    Ok(0)
}
