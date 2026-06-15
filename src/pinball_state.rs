// Important: This module will be used to
// define the structs for the game state as
// to stop using static/mutexes everywhere.
// This change will be done over the course of multiple commits iteratively.
// Until this notice is removed, it may not hold all possible states.

use crate::options::GameInput;

pub struct PinballState {
    pub options_state: OptionsState
}

impl PinballState {
    pub fn new() -> PinballState {
        Self {
            options_state: OptionsState::new()
        }
    }
}

pub struct OptionsState {
    pub show_dialog: bool,
    pub control_waiting_for_input: Option<GameInput>,
}

impl OptionsState {
    pub fn new() -> OptionsState {
        Self {
            show_dialog: false,
            control_waiting_for_input: None,
        }
    }
    
    pub fn update_show_dialog(&mut self, value: bool) {
        self.show_dialog = value;
    }
    
    pub fn update_control_waiting_for_input(&mut self, value: Option<GameInput>) {
        self.control_waiting_for_input = value;
    }
}