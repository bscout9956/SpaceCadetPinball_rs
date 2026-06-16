use crate::score::ScoreMessageFontType;

#[allow(non_snake_case)]
pub struct ScoreState {
    pub MSG_FONTP: Option<ScoreMessageFontType>,
}

impl ScoreState {
    pub fn new() -> Self {
        Self { MSG_FONTP: None }
    }
}
