use crate::options;
use crate::state::high_score_state::HighScoreState;
use std::collections::HashMap;

pub(crate) fn write(
    high_score_state: &mut HighScoreState,
    settings: &mut HashMap<String, String>,
) -> i32 {
    let mut check_sum = 0;
    let scores: Vec<String> = Vec::with_capacity(5);

    for position in 0..5 {
        let mut temp_string = String::new();
        let table = &high_score_state.high_score_table[position];
        temp_string = temp_string + &format!("{}", position);
        temp_string = temp_string + &format!("{}", table.name);
        temp_string = temp_string + &format!("{}", position);
        temp_string = temp_string + &format!("{}", table.score);

        for i in (0..table.name.len()).rev() {
            check_sum += table.name.as_bytes()[i];
        }
        check_sum += table.score as u8;
    }

    options::set_int("Verification", check_sum as i32, settings);
    return 0;
}

pub struct HighScore {
    pub name: &'static str,
    pub score: i32,
}

impl HighScore {
    pub(crate) fn default() -> Self {
        Self { name: "", score: 0 }
    }
}

pub struct HighScoreEntry {
    pub entry: HighScore,
    pub position: i32,
}

impl HighScoreEntry {
    pub(crate) fn default() -> HighScoreEntry {
        Self {
            entry: HighScore::default(),
            position: 0,
        }
    }
}

pub(crate) fn show_high_score_dialog(high_score_state: &mut HighScoreState) {
    high_score_state.show_dialog = true;
}
