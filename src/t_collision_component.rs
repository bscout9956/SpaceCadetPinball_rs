use crate::control::ComponentControl;
use crate::errors::LoaderError;
use crate::loader;
use crate::loader::{SpriteData, VisualStruct};
use crate::maths::*;
use crate::render::RenderSprite;
use crate::t_ball::TBall;
use crate::t_edge_segment::{TEdgeSegment, TEdgeSegmentBehavior};
use crate::t_pinball_component::{IPinballComponent, MessageCode, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use std::cell::Cell;
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
#[allow(non_snake_case)]
pub struct TCollisionComponent {
    pub base: TPinballComponent,
    pub edge_list: Vec<Rc<RefCell<TEdgeSegment>>>,
    pub elasticity: f32,
    pub smoothness: f32,
    pub boost: f32,
    pub threshold: f32,
    pub soft_hit_sound_id: i32,
    pub hard_hit_sound_id: i32,
    pub AABB: RectF,
}

pub trait TCollisionComponentBehavior {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &Vector2,
        distance: f32,
        edge: &TEdgeSegment,
    );
    fn field_effect(&mut self, ball: &TBall, vec_destination: &mut Vector2) -> i32;
    fn default_collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
    ) -> bool;
}

impl TCollisionComponent {
    pub fn new(
        table: Option<Rc<RefCell<TPinballTable>>>,
        group_index: i32,
        create_wall: bool,
    ) -> Rc<RefCell<Self>> {
        let base =
            TPinballComponent::new(table.clone().map(|t| Rc::downgrade(&t)), group_index, true);

        let mut visual = VisualStruct::default();

        let mut instance = Rc::new(RefCell::new(Self::default()));
        let mut inst = instance.borrow_mut();
        inst.base.active_flag = true;
        if inst.base.group_name.is_none() {
            inst.base.unused_base_flag = true;
        }
        if group_index <= 0 {
            loader::default_vsi(&mut visual);
        } else {
            loader::query_visual(group_index, 0, &mut visual);
            if create_wall {
                let offset: f32 = table.as_ref().unwrap().borrow_mut().collision_comp_offset;
                let float_array = loader::query_float_attribute_ptr(group_index, 0, 600);
                match float_array {
                    Ok(array_ptr) => TEdgeSegment::install_wall(),
                    Err(e) => {
                        panic!("failed to load float attr ptr {}", e);
                    }
                }
            }
        }

        if let Some(t) = table {
            t.borrow_mut().component_list.push(instance.clone());
        }

        instance
    }
}

impl TCollisionComponentBehavior for TCollisionComponent {
    fn collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &Vector2,
        distance: f32,
        edge: &TEdgeSegment,
    ) {
        //TODO: Undo borrow?
        if let Some(pinball_table) = &self.t_pinball_component.pinball_table
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

    fn field_effect(&mut self, ball: &TBall, vec_destination: &mut Vector2) -> i32 {
        0 // wow
    }

    fn default_collision(
        &mut self,
        ball: &mut TBall,
        next_position: &Vector2,
        direction: &mut Vector2,
    ) -> bool {
        // TODO: Undo borrow
        if let Some(pinball_table) = &self.t_pinball_component.pinball_table
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

impl IPinballComponent for TCollisionComponent {
    // TODO: Some remain unimplemented I suppose?? Maybe I should look into trait defaults
    fn sprite_set(index: i32) {
        todo!()
    }

    fn sprite_set_ball(index: i32, pos: Vector2i, depth: f32) {
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

    fn message(&mut self, code: MessageCode, value: f32) -> MessageCode {
        todo!()
    }
}
