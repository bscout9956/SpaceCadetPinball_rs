use std::collections::VecDeque;
use crate::high_score::{HighScore, HighScoreEntry};

pub struct HighScoreState {
    pub high_score_table: [HighScore; 5],
    pub dlg_enter_name: bool,
    pub dlg_data: HighScoreEntry,
    pub show_dialog: bool,
    pub score_queue: VecDeque<HighScoreEntry>,
}

impl HighScoreState {
    pub(crate) fn new() -> HighScoreState {
        Self {
            high_score_table: std::array::from_fn(|_| HighScore::default()),
            dlg_enter_name: false,
            dlg_data: HighScoreEntry::default(),
            show_dialog: false,
            score_queue: VecDeque::new(),
        }
    }
}
