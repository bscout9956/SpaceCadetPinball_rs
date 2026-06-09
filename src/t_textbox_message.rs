use crate::pb;
use std::sync::atomic::Ordering::SeqCst;

pub struct TTextBoxMessage {
    pub next_message: Option<Box<TTextBoxMessage>>,
    pub text: &'static str,
    pub time: f32,
    pub end_ticks: usize,
    pub low_priority: bool,
}

impl TTextBoxMessage {
    pub fn new(text: &'static str, time: f32, end_ticks: i32, low_priority: bool) -> Self {
        Self {
            next_message: None,
            time,
            end_ticks: pb::TIME_TICKS.load(SeqCst) + (time * 1000.0f32) as usize,
            low_priority,
            text,
        }
    }

    pub fn time_left(&self) -> f32 {
        (self.end_ticks - pb::TIME_TICKS.load(SeqCst)) as f32 * 0.001f32
    }

    pub fn refresh(&mut self, time: f32) {
        self.time = time;
        self.end_ticks = pb::TIME_TICKS.load(SeqCst) + (time * 1000.0f32) as usize;
    }
}
