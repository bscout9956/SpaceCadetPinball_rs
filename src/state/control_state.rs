use crate::control::ComponentInfo;
use crate::state::component_state::ComponentState;
use crate::t_pinball_table::TPinballTable;
use std::cell::RefCell;
use std::rc::Weak;

pub const CHEAT_LEN: usize = 11;

pub struct ControlState {
    pub table_g: Option<Weak<RefCell<TPinballTable>>>,
    pub score_components: Vec<ComponentInfo>,
    pub simple_components: Vec<ComponentInfo>,
    pub easy_mode: bool,
    pub table_unlimited_balls: bool,
    pub cheat_buffer: RefCell<[u8; CHEAT_LEN]>,
    pub component_state: ComponentState,
}

fn info(
    name: &'static str,
    control_func: ComponentControlCallback,
    score_count: u32,
    scores: &[i32],
) -> ComponentInfo {
    ComponentInfo {
        name,
        control: Rc::new(RefCell::new(ComponentControl {
            control_func,
            score_count,
            scores: scores.to_vec(),
        })),
    }
}

impl ControlState {
    pub fn new() -> ControlState {
        Self {
            table_g: None,
            score_components: vec![],
            easy_mode: false,
            table_unlimited_balls: false,
            cheat_buffer: RefCell::new([0; CHEAT_LEN]),
            component_state: ComponentState::default(),
        }
    }
}
