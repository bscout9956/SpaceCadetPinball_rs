use crate::t_edge_manager::FieldEffectType;
use crate::t_edge_segment::IEdgeSegment;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct TEdgeBox {
    pub edge_list: Vec<Rc<RefCell<dyn IEdgeSegment>>>,
    pub field_list: Vec<Rc<RefCell<FieldEffectType>>>,
}
