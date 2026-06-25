use crate::loader;
use crate::loader::VisualStruct;
use crate::maths::*;
use crate::t_ball::TBall;
use crate::t_edge_segment::{IEdgeSegment, TEdgeSegment};
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use std::any::Any;
use std::{cell::RefCell, rc::Rc};

#[derive(Default, Clone)]
#[allow(non_snake_case)]
pub struct TCollisionComponent {
    pub base: TPinballComponent,
    pub edge_list: Vec<Rc<RefCell<dyn IEdgeSegment>>>,
    pub elasticity: f32,
    pub smoothness: f32,
    pub boost: f32,
    pub threshold: f32,
    pub soft_hit_sound_id: i32,
    pub hard_hit_sound_id: i32,
    pub aabb: RectF,
}

pub trait ICollisionComponent {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
        distance: f32,
        edge: &TEdgeSegment,
        time_ticks: usize,
    );

    fn edge_list(&mut self) -> &mut Vec<Rc<RefCell<dyn IEdgeSegment>>>;

    fn field_effect(&mut self, ball: &TBall, vec_destination: &mut Vector2) -> i32 {
        return 0;
    }
    #[allow(non_snake_case)]
    fn set_AABB(&mut self, aabb: RectF);
    #[allow(non_snake_case)]
    fn get_AABB(&self) -> Option<RectF>;
}

use crate::message_code::MessageCode;
use crate::state::pinball_state::PinballState;
use std::ops::{Deref, DerefMut};
use std::rc::Weak;

impl Deref for TCollisionComponent {
    type Target = TPinballComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for TCollisionComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TCollisionComponent {
    pub fn new(
        table: Option<Weak<RefCell<TPinballTable>>>,
        group_index: i32,
        create_wall: bool,
        state: &mut PinballState,
    ) -> Rc<RefCell<Self>> {
        let base = TPinballComponent::new(table, group_index, true, &mut state.loader_state);

        let mut visual = VisualStruct::default();

        if group_index <= 0 {
            loader::default_vsi(&mut visual);
        } else {
            loader::query_visual(group_index, 0, &mut visual, state);
        }

        let mut instance_data = Self {
            base,
            threshold: visual.kicker.threshold,
            elasticity: visual.elasticity,
            smoothness: visual.smoothness,
            boost: visual.kicker.boost,
            hard_hit_sound_id: visual.kicker.hard_hit_sound_id,
            soft_hit_sound_id: visual.soft_hit_sound_id,
            edge_list: vec![],
            AABB: RectF {
                x_max: -10000.0,
                y_max: -10000.0,
                x_min: 10000.0,
                y_min: 10000.0,
            },
        };
        instance_data.active_flag.set(true);
        if (*instance_data).group_name.is_none() {
            (*instance_data).unused_base_flag.set(true);
        }

        let instance = Rc::new(RefCell::new(instance_data));

        if create_wall && group_index > 0 {
            if let Some(tbl) = &instance.borrow().base.pinball_table {
                let offset: f32 = tbl.upgrade().unwrap().borrow().collision_comp_offset;
                let float_array =
                    loader::query_float_attribute_ptr(group_index, 0, 600, &mut state.loader_state);
                match float_array {
                    Ok(array_ptr) => {
                        let weak_comp =
                            Rc::downgrade(&instance) as Weak<RefCell<dyn IPinballComponent>>;

                        TEdgeSegment::install_wall(
                            array_ptr,
                            weak_comp,
                            &instance.borrow().base.active_flag,
                            visual.collision_group as u32,
                            offset,
                            0,
                        )
                    }
                    Err(e) => {
                        panic!("failed to load float attr ptr {}", e);
                    }
                };
            }
        }

        instance
    }

    pub fn default_collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
    ) -> bool {
        // TODO: Undo borrow
        if let Some(pinball_table) = &self.base.pinball_table
            && let Some(upgraded_table) = pinball_table.upgrade()
        {
            let table = upgraded_table.borrow();
            if table.tilt_lock_flag {
                basic_collision(
                    ball,
                    next_position,
                    direction,
                    self.elasticity,
                    self.smoothness,
                    1_000_000_000.0,
                    0.0,
                );
                return false;
            }
        }
        let mut collision = false;
        let proj_speed = basic_collision(
            ball,
            next_position,
            direction,
            self.elasticity,
            self.smoothness,
            self.threshold,
            self.boost,
        );
        if proj_speed > self.threshold {
            // TODO: implement me
            // loader::play_sound(self.hard_hit_sound_id, ball, "TCollisionComponent1");
            collision = true;
        } else if proj_speed > 0.2 {
            // TODO: implement me

            // loader::play_sound(self.soft_hit_sound_id, ball, "TCollisionComponent2");
        }
        collision
    }
}

impl ICollisionComponent for TCollisionComponent {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
        distance: f32,
        edge: &TEdgeSegment,
        time_ticks: usize,
    ) {
        //TODO: Undo borrow?
        if let Some(pinball_table) = &self.base.pinball_table
            && let Some(upgraded_table) = pinball_table.upgrade()
        {
            let table = upgraded_table.borrow();

            if table.tilt_lock_flag {
                basic_collision(
                    ball,
                    next_position,
                    direction,
                    self.elasticity,
                    self.smoothness,
                    1000000000.0,
                    0.0,
                );
                return;
            }
        }

        let proj_speed = basic_collision(
            ball,
            next_position,
            direction,
            self.elasticity,
            self.smoothness,
            self.threshold,
            self.boost,
        );
        // TODO: Implement loader::play_sound
        // if proj_speed > self.threshold {
        //     loader::play_sound(self.hard_hit_sound_id, ball, "TCollisionComponent1");
        // } else if proj_speed > 0.2 {
        //     loader::play_sound(self.soft_hit_sound_id, ball, "TCollisionComponent2");
        // }
    }

    fn edge_list(&mut self) -> &mut Vec<Rc<RefCell<dyn IEdgeSegment>>> {
        &mut self.edge_list
    }

    fn field_effect(&mut self, ball: &TBall, vec_destination: &mut Vector2) -> i32 {
        0 // wow
    }
}

impl IPinballComponent for TCollisionComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn group_name(&self) -> Option<String> {
        match self.group_name.as_ref() {
            None => None,
            Some(name) => Some(name.borrow().to_string()),
        }
    }
    fn group_index(&self) -> i32 {
        self.base.group_index
    }

    // TODO: Some remain unimplemented I suppose?? Maybe I should look into trait defaults
    fn sprite_set(&mut self, index: i32) {
        todo!()
    }

    fn sprite_set_ball(&self, index: i32, pos: Vector2i, depth: f32) {
        todo!()
    }

    fn get_coordinates(&self) -> Vector2 {
        todo!()
    }

    fn get_scoring(&self, index: u32) -> i32 {
        todo!()
    }

    fn port_draw(&self) {
        for edge in &self.edge_list {
            let edge_borrow = edge.borrow();
            edge_borrow.port_draw();
        }
    }

    fn message(&mut self, code: MessageCode, value: f32) -> i32 {
        todo!()
    }
}
