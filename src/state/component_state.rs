use crate::t_light::TLight;
use crate::t_pinball_component::IPinballComponent;
use crate::t_textbox::TTextBox;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct TSound; //(Temporary)

impl TSound {
    pub(crate) fn play(
        &self,
        sound_source: Option<Rc<RefCell<dyn IPinballComponent>>>,
        info: &str,
    ) -> f32 {
        todo!()
    }
}

pub struct ComponentState {
    pub info_text_box: ComponentRef<TTextBox>,
    pub lite_62: ComponentRef<TLight>,
    pub soundwave7: ComponentRef<TSound>,
    pub soundwave28: ComponentRef<TSound>,
}

impl Default for ComponentState {
    // TODO: Do we really need the name?
    fn default() -> ComponentState {
        Self {
            info_text_box: ComponentRef::new("info_text_box"),
            lite_62: ComponentRef::new("lite_62"),
            soundwave7: ComponentRef::new("soundwave7"),
            soundwave28: ComponentRef::new("soundwave28"),
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
