use crate::maths::Vector2;
use crate::t_pinball_component::TPinballComponent;
use sdl2::libc::fopen;
use sdl2::sys::SDL_RWFromFile;
use sdl2::sys::mixer::{
    Mix_AllocateChannels, Mix_Chunk, Mix_FreeChunk, Mix_HaltChannel, Mix_LoadWAV_RW, Mix_Pause,
    Mix_PlayChannelTimed, Mix_Playing, Mix_Resume, Mix_Volume,
};
use std::cmp::Ordering;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::str::FromStr;

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

impl Sound {
    fn init(mix_open: bool, channels: i32, enable_flag: bool, volume: i32) -> Self {
        Self {
            mix_open,
            num_channels: channels,
            volume,
            enabled_flag: enable_flag,
            channels: Vec::new(), // TODO?
        }
    }

    pub fn enable(&mut self, enable_flag: bool) {
        self.enabled_flag = enable_flag;
        if self.mix_open && !enable_flag {
            unsafe {
                Mix_HaltChannel(-1);
            }
        }
    }

    pub fn activate(&self) {
        unsafe {
            if self.mix_open {
                Mix_Resume(-1);
            }
        }
    }

    pub fn deactivate(&self) {
        unsafe {
            if self.mix_open {
                Mix_Pause(-1);
            }
        }
    }

    pub fn close(&mut self) {
        self.enable(false);
        self.channels.clear();
    }

    pub fn play_sound(
        &mut self,
        wave: *mut Mix_Chunk,
        time: i32,
        sound_source: TPinballComponent,
        _info: &[u8], // Unused in the decomp
    ) {
        if self.mix_open && self.enabled_flag && !wave.is_null() {
            unsafe {
                if Mix_Playing(-1) == self.num_channels {
                    if let Some(min) = self.channels.iter().min_by_key(|ch| ch.timestamp) {
                        if let Some(oldest_channel) =
                            self.channels.iter().position(|ch| std::ptr::eq(ch, min))
                        {
                            Mix_HaltChannel(oldest_channel as i32);
                        }
                    }
                }

                let channel = Mix_PlayChannelTimed(-1, wave, 0, -1);
                if channel != -1 {
                    self.channels[channel as usize].timestamp = time;
                    // TODO: Implement options:: to continue
                }
            }
        }
    }

    pub fn load_wave_file(&self, lp_name: &str) -> *mut Mix_Chunk {
        if !self.mix_open {
            return std::ptr::null_mut();
        }

        let lp_name_c_raw = CString::new(lp_name).unwrap().into_raw();
        let mode_c_raw = CString::new("r").unwrap().into_raw();
        if let Some(wav_exists) = std::fs::exists(Path::new(lp_name)).ok() {
            if !wav_exists {
                return std::ptr::null_mut();
            }
            unsafe { Mix_LoadWAV_RW(SDL_RWFromFile(lp_name_c_raw, mode_c_raw), 1) }
        } else {
            std::ptr::null_mut()
        }
    }

    pub fn free_sound(&self, wave: *mut Mix_Chunk) {
        if self.mix_open && !wave.is_null() {
            unsafe {
                Mix_FreeChunk(wave);
            }
        }
    }

    pub fn set_channels(&mut self, mut channels: i32) {
        if channels <= 0 {
            channels = 8;
        }

        self.num_channels = channels;
        self.channels
            .resize_with(channels as usize, ChannelInfo::default);
        if self.mix_open {
            unsafe {
                Mix_AllocateChannels(self.num_channels);
            }
        }
        self.set_volume(self.volume);
    }

    pub fn set_volume(&mut self, volume: i32) {
        self.volume = volume;
        if self.mix_open {
            unsafe {
                Mix_Volume(-1, volume);
            }
        }
    }
}

pub(crate) fn freesound(p0: *const Mix_Chunk) {
    todo!()
}

pub(crate) fn load_wave_file(p0: String) -> *const Mix_Chunk {
    todo!()
}