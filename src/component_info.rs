use crate::control::ComponentControl;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ComponentInfo {
    pub name: &'static str,
    pub control: Rc<RefCell<ComponentControl>>,
}

pub const CONTROL_BUMP_SCORES1: [i32; 4] = [500, 1000, 1500, 2000];
pub const CONTROL_ROLL_SCORES1: [i32; 1] = [2000];
pub const CONTROL_BUMP_SCORES2: [i32; 4] = [1500, 2500, 3500, 4500];
pub const CONTROL_ROLL_SCORES2: [i32; 1] = [500];
pub const CONTROL_REBO_SCORE1: [i32; 1] = [500];
pub const CONTROL_ONEWAY4_SCORE1: [i32; 6] = [15000, 30000, 75000, 30000, 15000, 7500];
pub const CONTROL_RAMP_SCORE1: [i32; 1] = [5000];
pub const CONTROL_ROLL_SCORE1: [i32; 1] = [20000];
pub const CONTROL_ROLL_SCORE2: [i32; 2] = [5000, 25000];
pub const CONTROL_ROLL_SCORE3: [i32; 1] = [10000];
pub const CONTROL_ROLL_SCORE4: [i32; 1] = [500];
pub const CONTROL_FLAG_SCORE1: [i32; 2] = [500, 2500];
pub const CONTROL_KICKOUT_SCORE1: [i32; 5] = [10000, 0, 20000, 50000, 150000];
pub const CONTROL_SINK_SCORE1: [i32; 3] = [2500, 5000, 7500];
pub const CONTROL_TARGET_SCORE1: [i32; 2] = [500, 5000];
pub const CONTROL_TARGET_SCORE2: [i32; 3] = [1500, 10000, 50000];
pub const CONTROL_TARGET_SCORE3: [i32; 2] = [500, 1500];
pub const CONTROL_TARGET_SCORE4: [i32; 1] = [750];
pub const CONTROL_TARGET_SCORE5: [i32; 1] = [1000];
pub const CONTROL_TARGET_SCORE6: [i32; 1] = [750];
pub const CONTROL_TARGET_SCORE7: [i32; 1] = [750];
pub const CONTROL_ROLL_SCORE5: [i32; 1] = [10000];
pub const CONTROL_KICKOUT_SCORE2: [i32; 1] = [20000];
pub const CONTROL_KICKOUT_SCORE3: [i32; 1] = [50000];
