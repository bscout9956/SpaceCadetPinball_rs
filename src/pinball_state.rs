// Important: This module will be used to
// define the structs for the game state as
// to stop using static/mutexes everywhere.
// This change will be done over the course of multiple commits iteratively.
// Until this notice is removed, it may not hold all possible states.

use crate::options::GameInput;

pub struct PinballState {
    pub main_state: MainState,
    pub options_state: OptionsState,
}

pub struct MainState {
    pub b_quit: bool,
    pub mouse_down: bool,
    pub has_focus: bool,
    pub return_value: i32,
}

impl MainState {
    fn new() -> MainState {
        Self {
            b_quit: false,
            mouse_down: false,
            has_focus: true,
            return_value: 0,
        }
    }

    pub fn update_b_quit(&mut self, value: bool) {
        self.b_quit = value;
    }

    pub fn update_mouse_down(&mut self, value: bool) {
        self.mouse_down = value;
    }

    pub fn update_has_focus(&mut self, value: bool) {
        self.has_focus = value;
    }

    pub fn update_return(&mut self, value: i32) {
        self.return_value = value;
    }
}

impl PinballState {
    pub fn new() -> PinballState {
        Self {
            main_state: MainState::new(),
            options_state: OptionsState::new(),
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
