use crate::state::fullscrn_state::FullscrnState;
use crate::state::main_state::MainState;
use crate::state::options_state::OptionsState;
use crate::state::pb_game_state::PbGameState;
use crate::state::render_state::RenderState;

pub struct PinballState {
    pub main_state: MainState,
    pub pb_game_state: PbGameState,
    pub options_state: OptionsState,
    pub fullscrn_state: FullscrnState,
    pub render_state: RenderState,
}

impl PinballState {
    pub fn new() -> PinballState {
        Self {
            main_state: MainState::new(),
            pb_game_state: PbGameState::new(),
            options_state: OptionsState::new(),
            fullscrn_state: FullscrnState::new(),
            render_state: RenderState::new(),
        }
    }
}