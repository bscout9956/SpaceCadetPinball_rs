use crate::t_pinball_component::IPinballComponent;
use std::cell::RefCell;

use std::rc::Rc;

pub struct TSound;

impl TSound {
    pub(crate) fn play(
        &self,
        sound_source: Option<Rc<RefCell<dyn IPinballComponent>>>,
        info: &str,
    ) -> f32 {
        todo!()
    }
}
