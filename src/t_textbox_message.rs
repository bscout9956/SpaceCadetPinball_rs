#[derive(PartialEq, Clone)]
pub struct TTextBoxMessage {
    pub next_message: Option<Box<TTextBoxMessage>>,
    pub text: String,
    pub time: f32,
    pub end_ticks: usize,
    pub low_priority: bool,
}

impl TTextBoxMessage {
    pub fn new(text: &str, time: f32, low_priority: bool, time_ticks: usize) -> Self {
        Self {
            next_message: None,
            time,
            end_ticks: time_ticks + (time * 1000.0f32) as usize,
            low_priority,
            text: String::from(text),
        }
    }

    pub fn time_left(&self, time_ticks: usize) -> f32 {
        (self.end_ticks - time_ticks) as f32 * 0.001f32
    }

    pub fn refresh(&mut self, time: f32, time_ticks: usize) {
        self.time = time;
        self.end_ticks = time_ticks + (time * 1000.0f32) as usize;
    }
}
