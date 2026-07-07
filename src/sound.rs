use crate::maths;
use crate::maths::{Vector2, Vector3};
use crate::state::pinball_state::PinballState;
use crate::state::sound_state::SoundState;
use crate::t_pinball_component::IPinballComponent;
use sdl2::sys::SDL_RWFromFile;
use sdl2::sys::mixer::{
    Mix_AllocateChannels, Mix_Chunk, Mix_FreeChunk, Mix_HaltChannel, Mix_LoadWAV_RW, Mix_Pause,
    Mix_PlayChannelTimed, Mix_Playing, Mix_Resume, Mix_SetPosition, Mix_Volume,
};
use std::f32::consts::PI;
use std::ffi::{CString, NulError};
use std::ops::Rem;
use std::path::Path;
use thiserror::Error;

pub struct ChannelInfo {
    pub timestamp: i32,
    pub position: Vector2,
}

impl Default for ChannelInfo {
    fn default() -> Self {
        Self {
            timestamp: 0,
            position: Vector2 { x: 0.0, y: 0.0 },
        }
    }
}

struct Sound {
    // private fields
    num_channels: i32,
    enabled_flag: bool, // Defaults to false
    volume: i32,        // Defaults to MIN_MAX_VOLUME
    mix_open: bool,     // Defaults to false
    channels: Vec<ChannelInfo>,
}

pub fn enable(enable_flag: bool, state: &mut SoundState) {
    state.enabled_flag = enable_flag;
    if state.mix_open && !enable_flag {
        unsafe {
            Mix_HaltChannel(-1);
        }
    }
}

pub fn activate(state: &mut SoundState) {
    unsafe {
        if state.mix_open {
            Mix_Resume(-1);
        }
    }
}

pub fn deactivate(state: &mut SoundState) {
    unsafe {
        if state.mix_open {
            Mix_Pause(-1);
        }
    }
}

pub fn close(state: &mut SoundState) {
    enable(false, state);
    state.channels.clear();
}

pub fn play_sound(
    sound_state: &mut SoundState,
    wave: Option<Mix_Chunk>,
    time_ticks: usize,
    stereo: bool,
    edge_manager: Option<&TEdgeManager>,
    sound_source: Option<&dyn IPinballComponent>,
    _info: &str,
) {
    if sound_state.mix_open
        && sound_state.enabled_flag
        && let Some(mut wv) = wave
    {
        unsafe {
            if Mix_Playing(-1) == sound_state.num_channels
                && let Some(min) = sound_state.channels.iter().min_by_key(|ch| ch.timestamp)
                && let Some(oldest_channel) = sound_state
                    .channels
                    .iter()
                    .position(|ch| std::ptr::eq(ch, min))
            {
                Mix_HaltChannel(oldest_channel as i32);
            }

            let channel = Mix_PlayChannelTimed(-1, &raw mut wv, 0, -1);
            if channel != -1 {
                sound_state.channels[channel as usize].timestamp = time_ticks as i32;
                if stereo {
                    // Positional audio uses collision grid 2D coordinates normalized to [0, 1]
                    // Point (0, 0) is bottom left table corner; point (1, 1) is top right table corner.
                    // Z is defined as: 0 at table level, positive axis goes up from table surface.

                    // Get the source sound position.
                    // Sound without position are assumed to be at the center top of the table.
                    let sound_pos;

                    if let Some(source) = sound_source
                        && let Some(edge_manager) = edge_manager
                    {
                        let sound_pos_2d = source.get_coordinates(edge_manager);
                        sound_pos = Vector2 {
                            x: sound_pos_2d.x,
                            y: sound_pos_2d.y,
                        };
                    } else {
                        sound_pos = Vector2 {
                            x: 0.5f32,
                            y: 1.0f32,
                        };
                    }
                    sound_state.channels[channel as usize].position = sound_pos;

                    // Listener is positioned at the bottom center of the table,
                    // at 0.5 height, so roughly a table half minus length.
                    let player_pos = Vector3::new(0.5f32, 0.0f32, 0.5f32);
                    let sound_pos_vec3 = Vector3 {
                        x: sound_pos.x,
                        y: sound_pos.y,
                        z: 0.0f32,
                    };
                    let sound_dir = maths::vector_sub_vec3(&sound_pos_vec3, &player_pos);

                    // Find sound angle from positive Y axis in clockwise direction with atan2
                    // Remap atan2 output from (-Pi, Pi] to [0, 2 * Pi)
                    let angle = f32::rem(
                        f32::atan2(sound_dir.x, sound_dir.y) + PI * 2.0f32,
                        PI * 2.0f32,
                    );
                    let angle_deg = angle.to_degrees();
                    let angle_sdl = angle_deg as i16;

                    // Distance from listener to the sound position is roughly in the [0, ~1.22] range.
                    // Remap to [0, 122] by multiplying by 100 and cast to an integer.
                    let distance = (100.0f32 * maths::magnitude(&sound_dir)) as u8; // tf?

                    // Mix_SetPosition expects an angle in (Sint16)degrees, where
                    // angle 0 is due north, and rotates clockwise as the value increases.
                    // Mix_SetPosition expects a (Uint8)distance from 0 (near) to 255 (far).
                    Mix_SetPosition(channel, angle_sdl, distance);
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum SoundError {
    #[error(transparent)]
    NulError(#[from] NulError),
}

pub fn load_wave_file(
    lp_name: &str,
    state: &mut SoundState,
) -> Result<Option<Mix_Chunk>, SoundError> {
    if !state.mix_open {
        return Ok(None);
    }

    let c_string = CString::new(lp_name)?;
    let mode_c_raw = CString::new("r")?;
    if let Ok(wav_exists) = std::fs::exists(Path::new(lp_name)) {
        if !wav_exists {
            return Ok(None);
        }
        unsafe {
            let chunk = Mix_LoadWAV_RW(SDL_RWFromFile(c_string.as_ptr(), mode_c_raw.as_ptr()), 1);
            if chunk.is_null() {
                Ok(None)
            } else {
                Ok(Some(*chunk))
            }
        }
    } else {
        Ok(None)
    }
}

pub fn set_channels(state: &mut SoundState, mut channels: i32) {
    if channels <= 0 {
        channels = 8;
    }

    state.num_channels = channels;
    state
        .channels
        .resize_with(channels as usize, ChannelInfo::default);
    if state.mix_open {
        unsafe {
            Mix_AllocateChannels(state.num_channels);
        }
    }
    set_volume(state, state.volume);
}

pub fn set_volume(state: &mut SoundState, volume: i32) {
    state.volume = volume;
    if state.mix_open {
        unsafe {
            Mix_Volume(-1, volume);
        }
    }
}

pub fn free_sound(wave: Option<Mix_Chunk>, sound_state: &mut SoundState) {
    if sound_state.mix_open
        && let Some(mut wv) = wave
    {
        unsafe { Mix_FreeChunk(&raw mut wv) };
    }
}

pub(crate) fn init(
    mix_open: bool,
    channels: i32,
    enable_flag: bool,
    volume: i32,
    state: &mut SoundState,
) {
    state.mix_open = mix_open;
    state.volume = volume;
    set_channels(state, channels);
    enable(enable_flag, state);
}
