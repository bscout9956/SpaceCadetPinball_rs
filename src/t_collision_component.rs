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
    pub fn new(table: TPinballTable, group_index: i32, create_wall: bool) -> Self {
        // TODO: Should we pass the table or create a new one?
        let pinball_table = Rc::new(RefCell::new(TPinballTable::new()));
        let pinball_component = TPinballComponent::new(Some(pinball_table), group_index, true);

        let visual: VisualStruct = VisualStruct {
            smoothness: todo!(),
            elasticity: todo!(),
            float_arr_count: todo!(),
            float_arr: todo!(),
            soft_hit_sound_id: todo!(),
            kicker: todo!(),
            collision_group: todo!(),
            sound_index_4: todo!(),
            sound_index_3: todo!(),
            bitmap: todo!(),
        };

        let mut instance = Self {
            edge_list: vec![],
            elasticity: 0.0,
            smoothness: 0.0,
            boost: 0.0,
            threshold: 0.0,
            soft_hit_sound_id: 0,
            hard_hit_sound_id: 0,
            AABB: RectF {
                x_max: -10000.0,
                y_max: -10000.0,
                x_min: 10000.0,
                y_min: 10000.0,
            },
            t_pinball_component: pinball_component,
        };

        instance.t_pinball_component.active_flag = Rc::new(Cell::new(true));
        if instance.t_pinball_component.group_name.is_none() {
            instance.t_pinball_component.unused_base_flag = Rc::new(Cell::new(true));
        }
        // if group_index <= 0 {
        //     // TODO: Implement loader
        //     loader::default_vsi(visual);
        // } else {
        //     // TODO: Implement loader
        //     loader::query_visual(group_index, 0, &visual);
        //     if (create_wall) {
        //         let offset: f32 = table.collision_comp_offset;
        //         // TODO: Implement loader
        //         let float_arr = loader::query_float_attribute(group_index, 0, 600);
        //         TEdgeSegment::install_wall(
        //             &float_arr,
        //             &Rc::new(RefCell::new(Instance)),
        //             Instance.t_pinball_component.active_flag,
        //             visual.collision_group,
        //             offset,
        //             0,
        //         );
        //     }
        // }

        instance.threshold = visual.kicker.threshold;
        instance.elasticity = visual.elasticity;
        instance.smoothness = visual.smoothness;
        instance.boost = visual.kicker.boost;
        instance.hard_hit_sound_id = visual.kicker.hard_hit_sound_id;
        instance.soft_hit_sound_id = visual.soft_hit_sound_id;
        instance.t_pinball_component.group_index = group_index;

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
