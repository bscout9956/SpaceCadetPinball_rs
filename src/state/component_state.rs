use crate::t_blocker::TBlocker;
use crate::t_flipper::TFlipper;
use crate::t_light::TLight;
use crate::t_plunger::TPlunger;
use crate::t_sound::TSound;
use crate::t_textbox::TTextBox;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct ComponentState {
    pub info_text_box: ComponentRef<TTextBox>,
    pub lite_62: ComponentRef<TLight>,
    pub soundwave_7: ComponentRef<TSound>,
    pub soundwave_28: ComponentRef<TSound>,
    pub block_1: ComponentRef<TBlocker>,
    pub lite_1: ComponentRef<TLight>,
    pub lite_77: ComponentRef<TLight>,
    pub flip_1: ComponentRef<TFlipper>,
    pub flip_2: ComponentRef<TFlipper>,
    pub plunger: ComponentRef<TPlunger>,
}

impl Default for ComponentState {
    // TODO: Do we really need the name?
    fn default() -> ComponentState {
        Self {
            // TODO: I'll be damned if the names have to be exactly the same, don't think so tho
            block_1: ComponentRef::new("v_bloc1"),
            info_text_box: ComponentRef::new("info_text_box"),
            mission_text_box: ComponentRef::new("mission_text_box"),
            lite_1: ComponentRef::new("lite1"),
            lite_77: ComponentRef::new("lite77"),
            lite_62: ComponentRef::new("lite62"),
            soundwave_7: ComponentRef::new("soundwave7"),
            soundwave_28: ComponentRef::new("soundwave28"),
            flip_1: ComponentRef::new("a_flip1"),
            flip_2: ComponentRef::new("a_flip2"),
            plunger: ComponentRef::new("plunger"),
        }
    }
}

pub struct ComponentRef<T> {
    pub name: &'static str,
    // TODO: Do we want this as a Weak or an Rc?
    pub component: Option<Weak<RefCell<T>>>,
}

impl<T> ComponentRef<T> {
    pub fn new(name: &'static str) -> ComponentRef<T> {
        ComponentRef {
            // TODO: Do we really need the name?
            name,
            component: None,
        }
    }

    pub fn get(&self) -> Option<Rc<RefCell<T>>> {
        self.component.as_ref().and_then(Weak::upgrade)
    }

    pub fn set(&mut self, component: &Rc<RefCell<T>>) {
        self.component = Some(Rc::downgrade(component));
    }
}
