use std::{
    cell::{Cell, Ref, RefCell},
    ops::Deref,
    rc::{Rc, Weak},
};

use crate::{loader, maths::*};
use crate::{loader::SpriteData, t_pinball_table::TPinballTable};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct MessageCode(pub i32);

#[allow(dead_code)]
impl MessageCode {
    // Private codes <1000, different meaning for each component
    pub const T_FLIPPER_NULL: MessageCode = MessageCode(0);
    pub const T_FLIPPER_EXTEND: MessageCode = MessageCode(1);
    pub const T_FLIPPER_RETRACT: MessageCode = MessageCode(2);

    pub const T_LIGHT_TURN_OFF: MessageCode = MessageCode(0);
    pub const T_LIGHT_TURN_ON: MessageCode = MessageCode(1);
    pub const T_LIGHT_GET_LIGHT_ON_FLAG: MessageCode = MessageCode(2);
    pub const T_LIGHT_GET_FLASHER_ON_FLAG: MessageCode = MessageCode(3);
    pub const T_LIGHT_FLASHER_START: MessageCode = MessageCode(4);
    pub const T_LIGHT_APPLY_MULT_DELAY: MessageCode = MessageCode(5);
    pub const T_LIGHT_APPLY_DELAY: MessageCode = MessageCode(6);
    pub const T_LIGHT_FLASHER_START_TIMED: MessageCode = MessageCode(7);
    pub const T_LIGHT_TURN_OFF_TIMED: MessageCode = MessageCode(8);
    pub const T_LIGHT_TURN_ON_TIMED: MessageCode = MessageCode(9);
    pub const T_LIGHT_SET_ON_STATE_BMP_INDEX: MessageCode = MessageCode(11);
    pub const T_LIGHT_INC_ON_STATE_BMP_INDEX: MessageCode = MessageCode(12);
    pub const T_LIGHT_DEC_ON_STATE_BMP_INDEX: MessageCode = MessageCode(13);
    pub const T_LIGHT_RESET_TIMED: MessageCode = MessageCode(14);
    pub const T_LIGHT_FLASHER_START_TIMED_THEN_STAY_ON: MessageCode = MessageCode(15);
    pub const T_LIGHT_FLASHER_START_TIMED_THEN_STAY_OFF: MessageCode = MessageCode(16);
    pub const T_LIGHT_TOGGLE_VALUE: MessageCode = MessageCode(17);
    pub const T_LIGHT_RESET_AND_TOGGLE_VALUE: MessageCode = MessageCode(18);
    pub const T_LIGHT_RESET_AND_TURN_ON: MessageCode = MessageCode(19);
    pub const T_LIGHT_RESET_AND_TURN_OFF: MessageCode = MessageCode(20);
    pub const T_LIGHT_TOGGLE: MessageCode = MessageCode(21);
    pub const T_LIGHT_RESET_AND_TOGGLE: MessageCode = MessageCode(22);
    pub const T_LIGHT_SET_MESSAGE_FIELD: MessageCode = MessageCode(23);
    pub const T_LIGHT_FT_TMP_OVERRIDE_ON: MessageCode = MessageCode(-24);
    pub const T_LIGHT_FT_TMP_OVERRIDE_OFF: MessageCode = MessageCode(-25);
    pub const T_LIGHT_FT_RESET_OVERRIDE: MessageCode = MessageCode(-26);

    pub const T_LIGHT_GROUP_NULL: MessageCode = MessageCode(0);
    pub const T_LIGHT_GROUP_STEP_BACKWARD: MessageCode = MessageCode(24);
    pub const T_LIGHT_GROUP_STEP_FORWARD: MessageCode = MessageCode(25);
    pub const T_LIGHT_GROUP_ANIMATION_BACKWARD: MessageCode = MessageCode(26);
    pub const T_LIGHT_GROUP_ANIMATION_FORWARD: MessageCode = MessageCode(27);
    pub const T_LIGHT_GROUP_LIGHT_SHOW_ANIMATION: MessageCode = MessageCode(28);
    pub const T_LIGHT_GROUP_GAME_OVER_ANIMATION: MessageCode = MessageCode(29);
    pub const T_LIGHT_GROUP_RANDOM_ANIMATION_SATURATION: MessageCode = MessageCode(30);
    pub const T_LIGHT_GROUP_RANDOM_ANIMATION_DESATURATION: MessageCode = MessageCode(31);
    pub const T_LIGHT_GROUP_OFFSET_ANIMATION_FORWARD: MessageCode = MessageCode(32);
    pub const T_LIGHT_GROUP_OFFSET_ANIMATION_BACKWARD: MessageCode = MessageCode(33);
    pub const T_LIGHT_GROUP_RESET: MessageCode = MessageCode(34);
    pub const T_LIGHT_GROUP_TURN_ON_AT_INDEX: MessageCode = MessageCode(35);
    pub const T_LIGHT_GROUP_TURN_OFF_AT_INDEX: MessageCode = MessageCode(36);
    pub const T_LIGHT_GROUP_GET_ON_COUNT: MessageCode = MessageCode(37);
    pub const T_LIGHT_GROUP_GET_LIGHT_COUNT: MessageCode = MessageCode(38);
    pub const T_LIGHT_GROUP_GET_MESSAGE2: MessageCode = MessageCode(39);
    pub const T_LIGHT_GROUP_GET_ANIMATION_FLAG: MessageCode = MessageCode(40);
    pub const T_LIGHT_GROUP_RESET_AND_TURN_ON: MessageCode = MessageCode(41);
    pub const T_LIGHT_GROUP_RESET_AND_TURN_OFF: MessageCode = MessageCode(42);
    pub const T_LIGHT_GROUP_RESTART_NOTIFY_TIMER: MessageCode = MessageCode(43);
    pub const T_LIGHT_GROUP_FLASH_WHEN_ON: MessageCode = MessageCode(44);
    pub const T_LIGHT_GROUP_TOGGLE_SPLIT_INDEX: MessageCode = MessageCode(45);
    pub const T_LIGHT_GROUP_START_FLASHER: MessageCode = MessageCode(46);
    pub const T_LIGHT_GROUP_COUNTDOWN_ENDED: MessageCode = MessageCode(47);

    pub const T_BUMPER_SET_BMP_INDEX: MessageCode = MessageCode(11);
    pub const T_BUMPER_INC_BMP_INDEX: MessageCode = MessageCode(12);
    pub const T_BUMPER_DEC_BMP_INDEX: MessageCode = MessageCode(13);

    pub const T_COMPONENT_GROUP_RESET_NOTIFY_TIMER: MessageCode = MessageCode(48);

    pub const T_POPUP_TARGET_DISABLE: MessageCode = MessageCode(49);
    pub const T_POPUP_TARGET_ENABLE: MessageCode = MessageCode(50);

    pub const T_BLOCKER_DISABLE: MessageCode = MessageCode(51);
    pub const T_BLOCKER_ENABLE: MessageCode = MessageCode(52);
    pub const T_BLOCKER_RESTART_TIMEOUT: MessageCode = MessageCode(59);

    pub const T_GATE_DISABLE: MessageCode = MessageCode(53);
    pub const T_GATE_ENABLE: MessageCode = MessageCode(54);

    pub const T_KICKOUT_RESTART_TIMER: MessageCode = MessageCode(55);

    pub const T_SINK_UNKNOWN7: MessageCode = MessageCode(7);
    pub const T_SINK_RESET_TIMER: MessageCode = MessageCode(56);

    pub const T_SOLO_TARGET_DISABLE: MessageCode = MessageCode(49);
    pub const T_SOLO_TARGET_ENABLE: MessageCode = MessageCode(50);

    pub const T_TIMER_RESET_TIMER: MessageCode = MessageCode(59);

    pub const CONTROL_BALL_CAPTURED: MessageCode = MessageCode(57);
    pub const CONTROL_BALL_RELEASED: MessageCode = MessageCode(58);
    pub const CONTROL_TIMER_EXPIRED: MessageCode = MessageCode(60);
    pub const CONTROL_NOTIFY_TIMER_EXPIRED: MessageCode = MessageCode(61);
    pub const CONTROL_SPINNER_LOOP_RESET: MessageCode = MessageCode(62);
    pub const CONTROL_COLLISION: MessageCode = MessageCode(63);
    pub const CONTROL_ENABLE_MULTIPLIER: MessageCode = MessageCode(64);
    pub const CONTROL_DISABLE_MULTIPLIER: MessageCode = MessageCode(65);
    pub const CONTROL_MISSION_COMPLETE: MessageCode = MessageCode(66);
    pub const CONTROL_MISSION_STARTED: MessageCode = MessageCode(67);

    // Public codes 1000+, apply to all components
    pub const LEFT_FLIPPER_INPUT_PRESSED: MessageCode = MessageCode(1000);
    pub const LEFT_FLIPPER_INPUT_RELEASED: MessageCode = MessageCode(1001);
    pub const RIGHT_FLIPPER_INPUT_PRESSED: MessageCode = MessageCode(1002);
    pub const RIGHT_FLIPPER_INPUT_RELEASED: MessageCode = MessageCode(1003);
    pub const PLUNGER_INPUT_PRESSED: MessageCode = MessageCode(1004);
    pub const PLUNGER_INPUT_RELEASED: MessageCode = MessageCode(1005);
    pub const PAUSE: MessageCode = MessageCode(1008);
    pub const RESUME: MessageCode = MessageCode(1009);
    pub const LOOSE_FOCUS: MessageCode = MessageCode(1010);
    pub const SET_TILT_LOCK: MessageCode = MessageCode(1011);
    pub const CLEAR_TILT_LOCK: MessageCode = MessageCode(1012);
    pub const START_GAME_PLAYER1: MessageCode = MessageCode(1013);
    pub const NEW_GAME: MessageCode = MessageCode(1014);
    pub const PLUNGER_FEED_BALL: MessageCode = MessageCode(1015);
    pub const PLUNGER_START_FEED_TIMER: MessageCode = MessageCode(1016);
    pub const PLUNGER_LAUNCH_BALL: MessageCode = MessageCode(1017);
    pub const PLUNGER_RELAUNCH_BALL: MessageCode = MessageCode(1018);
    pub const PLAYER_CHANGED: MessageCode = MessageCode(1020);
    pub const SWITCH_TO_NEXT_PLAYER: MessageCode = MessageCode(1021);
    pub const GAME_OVER: MessageCode = MessageCode(1022);
    pub const RESET: MessageCode = MessageCode(1024);
}

// TODO: Temporary
struct Control;
// TODO: Temporary
struct RenderSprite;

pub struct TPinballComponent<'a> {
    pub unused_base_flag: Rc<Cell<bool>>,
    pub active_flag: Rc<Cell<bool>>,
    pub message_field: MessageCode,
    pub group_name: Option<Rc<Cell<String>>>,
    pub component_control: Option<Weak<RefCell<Control>>>,
    pub group_index: i32,
    pub render_sprite: RenderSprite, //TODO: Decide what this will be
    pub pinball_table: Option<Weak<RefCell<TPinballTable<'a>>>>,
    pub list_bitmap: Vec<SpriteData>,

    visual_pos_norm_x: f32,
    visual_pos_norm_y: f32,
}

pub trait TPinballComponentBehavior {
    fn sprite_set(index: i32);
    fn sprite_set_ball(index: i32, pos: Vector2i, depth: f32);
    fn get_coordinates(&self) -> Vector2;
    fn get_scoring(&self, index: u32) -> i32;
    fn port_draw(&self);
    fn message(&mut self, code: MessageCode, value: f32) -> MessageCode;
}

// TODO: Temporary
struct VisualStruct;

impl<'a> TPinballComponent<'a> {
    pub fn new(
        table: Option<Rc<RefCell<TPinballTable<'a>>>>,
        group_index: i32,
        load_visuals: bool,
    ) -> Self {
        let visual: VisualStruct;

        let mut instance = Self {
            unused_base_flag: Rc::new(Cell::new(false)),
            active_flag: Rc::new(Cell::new(false)),
            message_field: MessageCode(0),
            group_name: Some(Rc::new(Cell::new(String::new()))),
            component_control: None,
            group_index,
            render_sprite: RenderSprite,
            pinball_table: None,
            list_bitmap: Vec::new(),
            visual_pos_norm_x: -1.0,
            visual_pos_norm_y: -1.0,
        };

        if let Some(table_result) = table {
            let table_instance = Rc::downgrade(&table_result);
            instance.pinball_table = Some(table_instance)
            // TODO: table.component_list.push_back();
        }

        if group_index >= 0 {
            // TODO: Create module
            //instance.group_name = loader::query_name(group_index);
        }

        if load_visuals && group_index >= 0 {
            // TODO: Create module
            //let visual_count = loader::query_visual_states(group_index);
            // TODO: For loop L#33...
        }

        instance
    }
}

impl Drop for TPinballComponent<'_> {
    fn drop(&mut self) {
        if self.pinball_table.is_some() {
            // TODO: Add field, use let Some, upgrade weakptr, borrow mut?
            //let components = self.pinball_table.unwrap().component_list;
            //TODO: Implement component list first then let position = // std::find begin end for this
            // Continue from L94
        }

        // TODO: We don't need this
        //drop(self.list_bitmap); // TODO: Does list_bitmap implement drop?
    }
}

impl TPinballComponentBehavior for TPinballComponent<'_> {
    fn sprite_set(index: i32) {
        todo!()
    }

    fn sprite_set_ball(index: i32, pos: Vector2i, depth: f32) {
        todo!()
    }

    fn get_coordinates(&self) -> Vector2 {
        Vector2 {
            x: self.visual_pos_norm_x,
            y: self.visual_pos_norm_y,
        }
    }

    fn get_scoring(&self, index: u32) -> i32 {
        if let Some(weak_control) = &self.component_control
            && let Some(strong_control_rc) = weak_control.upgrade()
        {
            let control = strong_control_rc.borrow();

            // TODO: Implement control struct
            // if index >= control.score_count {
            //     0
            // } else {
            //     control.scores[index as usize]
            // }
        }
        0
    }

    fn port_draw(&self) {
        // Doesn't have an impl
    }

    fn message(&mut self, code: MessageCode, value: f32) -> MessageCode {
        self.message_field = code;
        if code == MessageCode::RESET {
            self.message_field = MessageCode(0);
        }

        MessageCode(0)
    }
}
