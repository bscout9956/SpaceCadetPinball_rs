use crate::sound::ChannelInfo;

#[derive(Default)]
pub struct SoundState {
    pub num_channels: i32,
    pub enabled_flag: bool,
    pub volume: i32,
    pub mix_open: bool,
    pub channels: Vec<ChannelInfo>
}
