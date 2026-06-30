// Important: This module will be used to
// define the structs for the game state as
// to stop using static/mutexes everywhere.
// This change will be done over the course of multiple commits iteratively.
// Until this notice is removed, it may not hold all possible states.

pub mod component_state;
pub mod control_state;
pub mod debug_state;
pub mod fullscrn_state;
pub mod high_score_state;
pub mod loader_state;
pub mod main_state;
pub mod options_state;
pub mod pb_game_state;
pub mod pinball_state;
pub mod render_state;
pub mod score_state;
pub mod sound_state;
