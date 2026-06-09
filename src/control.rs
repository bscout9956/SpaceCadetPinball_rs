use crate::t_pinball_component::{MessageCode, TPinballComponent};

pub(crate) fn table_control_handler(p0: MessageCode) {
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

struct ComponentInfo {
    tag: ComponentTagBase,
    control: ComponentControl,
}

