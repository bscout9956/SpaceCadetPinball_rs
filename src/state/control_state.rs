use crate::t_pinball_table::TPinballTable;
use std::cell::RefCell;
use std::rc::Weak;
use crate::control::ComponentInfo;

pub struct ControlState {
    pub table_g: Option<Weak<RefCell<TPinballTable>>>,
    pub score_components: Vec<ComponentInfo>,
    pub easy_mode: bool,
    pub table_unlimited_balls: bool
}

impl ControlState {
    pub fn new() -> ControlState {
        Self { table_g: None, score_components: vec![], easy_mode: false, table_unlimited_balls: false }
    }
}
