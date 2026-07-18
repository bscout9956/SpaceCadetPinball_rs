use crate::context::component_context::ComponentContext;
use crate::control::ComponentControl;
use crate::loader;
use crate::loader::VisualStruct;
use crate::maths::{RectF, Vector2};
use crate::render::RenderSpriteRef;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_collision_component::{ICollisionComponent, TCollisionComponent};
use crate::t_edge_manager::TEdgeManager;
use crate::t_edge_segment::IEdgeSegment;
use crate::t_line::TLine;
use crate::t_pinball_component::IPinballComponent;
use crate::t_pinball_table::TPinballTable;
use anyhow::Result;
use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct TOneWay {
    pub base: TCollisionComponent,
    pub line: Option<Rc<RefCell<TLine>>>,
}

impl TOneWay {
    pub(crate) fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        state: &mut PinballState,
    ) -> Result<Rc<RefCell<TOneWay>>> {
        let base = TCollisionComponent::new(table.clone(), group_index, false, state)?;
        let mut visual = VisualStruct::default();
        let mut line_pt_1 = Vector2::default();
        let mut line_pt_2 = Vector2::default();

        let mut instance = Self {
            base: base.take(),
            line: None,
        };

        let instance_rc = Rc::new(RefCell::new(instance));
        let collision_rc: Rc<RefCell<dyn ICollisionComponent>> = instance_rc.clone();
        let weak_inst: Weak<RefCell<dyn ICollisionComponent>> = Rc::downgrade(&collision_rc);

        loader::query_visual(group_index, 0, &mut visual, state)?;

        if visual.float_arr_count == 2 {
            line_pt_2.x = visual.float_arr[0];
            line_pt_2.y = visual.float_arr[1];
            line_pt_1.x = visual.float_arr[2];
            line_pt_1.y = visual.float_arr[3];

            let line = TLine::new_vec2(
                Some(weak_inst.clone()),
                instance_rc.borrow().base.active_flag.clone(),
                visual.collision_group as u32,
                &line_pt_2,
                &line_pt_1,
            );
            let mut line_rc = Rc::new(RefCell::new(line));

            if let Some(t_shit) = table.as_ref() {
                if let Some(table) = t_shit.upgrade() {
                    line_rc
                        .borrow_mut()
                        .offset(table.borrow().collision_comp_offset);
                    line_rc.borrow_mut().place_in_grid(
                        &mut instance_rc.borrow_mut().base.aabb,
                        Some(line_rc.clone()),
                        &mut state.pb_game_state,
                    )?;
                    instance_rc
                        .borrow_mut()
                        .base
                        .edge_list
                        .push(line_rc.clone());

                    
                    let line_2 = TLine::new_vec2(
                        Some(weak_inst),
                        instance_rc.borrow().base.active_flag.clone(),
                        visual.collision_group as u32,
                        &line_pt_1,
                        &line_pt_2,
                    );
                    line_rc = Rc::new(RefCell::new(line_2));
                    instance_rc.borrow_mut().line = Some(line_rc.clone());

                    line_rc
                        .borrow_mut()
                        .offset(-table.borrow().collision_comp_offset * 0.8f32);

                    line_rc.borrow().place_in_grid(
                        &mut instance_rc.borrow_mut().base.aabb,
                        Some(line_rc.clone()),
                        &mut state.pb_game_state,
                    )?;

                    // TODO: Push this second line into base.edge_list as
                    // well.
                }
            }
        }

        Ok(instance_rc)
    }
}

// TODO: Delegate the component plumbing below to `base`, following
// TWall/TGate
impl IPinballComponent for TOneWay {
    fn render_sprite(&self) -> Option<RenderSpriteRef> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        todo!()
    }

    fn group_name(&self) -> Option<Rc<RefCell<String>>> {
        todo!()
    }

    fn group_index(&self) -> i32 {
        todo!()
    }

    fn sprite_set(&mut self, index: i32) {
        todo!()
    }

    fn get_coordinates(&self, edge_manager: &TEdgeManager) -> Vector2 {
        todo!()
    }

    fn get_scoring(&self, index: u32) -> i32 {
        todo!()
    }

    fn port_draw(&self) {
        todo!()
    }

    fn set_active_flag(&mut self, active: bool) {
        todo!()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        todo!()
    }

    fn set_control(&mut self, control: Option<Weak<RefCell<ComponentControl>>>) {
        todo!()
    }
}

impl ICollisionComponent for TOneWay {
    fn collision(
        &mut self,
        ball: &mut Rc<RefCell<TBall>>,
        next_position: &Vector2,
        direction: &mut Vector2,
        distance: f32,
        edge: &dyn IEdgeSegment,
        component_context: &mut ComponentContext,
    ) -> Result<()> {
        // TODO: Port TOneway::Collision from the original:
        todo!()
    }

    fn edge_list(&mut self) -> &mut Vec<Rc<RefCell<dyn IEdgeSegment>>> {
        // TODO: Delegate to self.base.edge_list().
        todo!()
    }

    fn set_AABB(&mut self, aabb: RectF) {
        // TODO: Delegate to self.base.set_AABB(aabb).
        todo!()
    }

    fn get_AABB(&self) -> Option<RectF> {
        // TODO: Delegate to self.base.get_AABB().
        todo!()
    }
}
