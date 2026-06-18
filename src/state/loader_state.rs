use crate::group_data::DatFile;
use crate::loader::SoundListStruct;
use std::sync::{Arc, RwLock};

pub struct LoaderState {
    pub sound_count: i32,
    pub loader_sound_count: i32,
    pub loader_table: Option<Arc<RwLock<DatFile>>>,
    pub sound_record_table: Option<Arc<RwLock<DatFile>>>,
    pub sound_list: [SoundListStruct; 65],
}

impl LoaderState {
    pub fn new() -> Self {
        Self {
            sound_count: 1,
            loader_sound_count: 0,
            loader_table: None,
            sound_record_table: None,
            sound_list: std::array::from_fn(|_| SoundListStruct::default()),
        }
    }
}
