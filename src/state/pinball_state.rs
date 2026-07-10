use crate::context::component_context::ComponentContext;
use crate::state::control_state::ControlState;
use crate::state::debug_state::DebugState;
use crate::state::fullscrn_state::FullscrnState;
use crate::state::high_score_state::HighScoreState;
use crate::state::loader_state::LoaderState;
use crate::state::main_state::MainState;
use crate::state::options_state::OptionsState;
use crate::state::pb_game_state::PbGameState;
use crate::state::render_state::RenderState;
use crate::state::score_state::ScoreState;
use crate::state::sound_state::SoundState;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PinballState {
    pub main_state: MainState,
    pub pb_game_state: PbGameState,
    pub options_state: OptionsState,
    pub fullscrn_state: FullscrnState,
    pub render_state: RenderState,
    pub loader_state: LoaderState,
    pub high_score_state: HighScoreState,
    pub sound_state: SoundState,
    pub score_state: ScoreState,
    pub control_state: ControlState,
    pub debug_state: DebugState,
    pub timer_manager: Rc<RefCell<TimerManager>>,
}

impl PinballState {
    pub fn new() -> PinballState {
        let timer_rc = TimerManager::default();
        Self {
            main_state: MainState::new(),
            pb_game_state: PbGameState::default(),
            options_state: OptionsState::default(),
            fullscrn_state: FullscrnState::default(),
            render_state: RenderState::default(),
            loader_state: LoaderState::new(),
            high_score_state: HighScoreState::new(),
            sound_state: SoundState::default(),
            score_state: ScoreState::new(),
            control_state: ControlState::new(),
            debug_state: DebugState::new(),
            timer_manager: Rc::new(RefCell::new(timer_rc)),
        }
    }

    pub fn get_component_context(&mut self) -> ComponentContext<'_> {
        ComponentContext::from_state(self)
    }
}

impl Default for PinballState {
    fn default() -> Self {
        Self::new()
    }
}
