use std::cell::RefCell;
use std::rc::Weak;
use crate::message_code::MessageCode;
use crate::t_ball::TBall;
use crate::t_pinball_component::TPinballComponent;
use crate::t_pinball_table::TPinballTable;

pub fn table_control_handler(code: MessageCode) {
    todo!()
}

struct ComponentTagBase {
    name: &'static str,
}

impl ComponentTagBase {
    fn new(name: &'static str) -> Self {
        Self { name }
    }
}

pub trait ComponentTagBaseBehavior {
    fn get_component(&self) -> Option<TPinballComponent>;
    fn set_component(comp: Option<TPinballComponent>);
}

pub struct ComponentControl {
    pub score_count: u32,
    pub scores: Vec<i32>,
}

pub struct ComponentInfo {
    tag: ComponentTagBase,
    control: ComponentControl,
}

pub(crate) fn pbctrl_bdoor_controller(p0: &u8) {
    todo!()
}

pub(crate) fn unstuck_ball(p0: &mut TBall, p1: usize) {
    todo!()
}

pub(crate) fn cheat_bump_rank() {
    todo!()
}

pub(crate) fn make_links(p0: Option<Weak<RefCell<TPinballTable>>>) {
    todo!()
}