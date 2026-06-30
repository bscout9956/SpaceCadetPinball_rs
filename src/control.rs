use crate::message_code::MessageCode;
use crate::pb;
use crate::state::component_state::ComponentState;
use crate::state::control_state::{CHEAT_LEN, ControlState};
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::translations::Msg;
use anyhow::Result;
use std::cell::RefCell;
use std::rc::Rc;

pub fn table_control_handler(code: MessageCode) {
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

pub struct ComponentInfo {
    tag: ComponentTagBase,
    control: ComponentControl,
}

pub(crate) fn pbctrl_bdoor_controller(key: u8, state: &mut PinballState) -> Result<()> {
    const QUOTES: [&str; 8] = [
        "Hey, is that a screen saver?",
        "I guess it has been a good week",
        "She may already be a glue bottle",
        "If you don't come in Saturday,\n...\n",
        "don't even bother coming in Sunday.",
        "Tomorrow already sucks",
        "I knew it worked too good to be right.",
        "World's most expensive flippers",
    ];

    const CREDITS: [&str; 35] = [
        "Full Tilt! was created by Cinematronics",
        "for Maxis.",
        "Cinematronics Team:",
        "Programming\nMichael Sandige\nJohn Taylor",
        "Art\nJohn Frantz Jr.\nRyan Medeiros",
        "Design\nKevin Gliner",
        "Sound Effects\nMatt Ridgeway",
        "Donald S. Griffin",
        "Design Consultant\nMark Sprenger",
        "Music\nMatt Ridgeway",
        "Producer\nKevin Gliner",
        "Voices\nMike McGeary\nWilliam Rice",
        "Grand Poobah\nDavid Stafford",
        "Special Thanks\nPaula Sandige\nAlex St. John",
        "Brad Silverberg\nJeff Camp",
        "Danny Thorpe\nGreg Hospelhorn",
        "Maxis Team:",
        "Producer\nJohn Csicsery",
        "Product Manager\nLarry Lee",
        "Lead Tester\nMichael Gilmartin",
        "QA Manager\nAlan Barton",
        "Additional Testing\nJoe Longworth\nScott Shicoff",
        "Owen Nelson\nJohn \"Jussi\" Ylinen",
        "John Landes\nMarc Meyer",
        "Cathy Castro\nKeith Meyer",
        "Additional Art\nOcean Quigley",
        "Rick Macaraeg\nCharlie Aquilina",
        "Art Director\nSharon Barr",
        "Install Program\nKevin O'Hare",
        "Intro Music",
        "Brian Conrad",
        "John Csicsery",
        "Special Thanks\nSam Poole\nJoe Scirica",
        "Jeff Braun\nBob Derber\nAshley Csicsery",
        "Tom Forge\nWill \"Burr\" Wright",
    ];

    state
        .control_state
        .cheat_buffer
        .borrow_mut()
        .copy_within(1..CHEAT_LEN, 0);
    state.control_state.cheat_buffer.borrow_mut()[CHEAT_LEN - 1] = key;

    if state
        .control_state
        .cheat_buffer
        .borrow_mut()
        .ends_with("hidden test".as_ref())
        || state
            .control_state
            .cheat_buffer
            .borrow_mut()
            .ends_with("hidden\ttest".as_ref())
    {
        state.pb_game_state.cheat_mode ^= true;
    } else if state
        .control_state
        .cheat_buffer
        .borrow_mut()
        .ends_with("gmax".as_ref())
    {
        gravity_well_kickout_control(MessageCode::CONTROL_ENABLE_MULTIPLIER, None, state)?;
    } else if state
        .control_state
        .cheat_buffer
        .borrow_mut()
        .ends_with("1max".as_ref())
    {
        state.pb_game_state.increment_table_balls();
        table_add_extra_ball(2.0f32, state)?;
    }

    Ok(())

    // TODO: todo!()
}

fn table_add_extra_ball(count: f32, state: &mut PinballState) -> Result<()> {
    if let Some(wave) = state.control_state.component_state.soundwave28.get() {
        wave.borrow().play(None, "table_add_extra_ball");
    }
    if let Some(itb) = state.control_state.component_state.info_text_box.get() {
        let rc_string = pb::get_rc_string(Msg::STRING110)?;
        itb.borrow_mut().display(rc_string, count, state, None)?;
    }
    Ok(())
}

fn gravity_well_kickout_control(
    code: MessageCode,
    caller: Option<Rc<RefCell<dyn IPinballComponent>>>,
    state: &mut PinballState,
) -> Result<()> {
    match code {
        MessageCode::CONTROL_COLLISION => {
            if let Some(t) = state.pb_game_state.main_table.as_ref()
                && let Some(c) = caller.as_ref()
                && let Some(tb) = state.control_state.component_state.info_text_box.get()
                && let Some(lite62) = state.control_state.component_state.lite_62.get()
                && let Some(soundwave7) = state.control_state.component_state.soundwave7.get()
            {
                let scoring = c.borrow().get_scoring(0);
                let added_score = t
                    .borrow_mut()
                    .add_score(scoring, state.pb_game_state.full_tilt_mode)?;

                let rc_string = pb::get_rc_string(Msg::STRING182)?
                    .replace("%ld", added_score.to_string().as_str());
                tb.borrow_mut().display(&rc_string, 2.0, state, None)?;
                lite62
                    .borrow_mut()
                    .message(MessageCode::T_LIGHT_RESET_AND_TURN_OFF, 0.0f32);
                c.borrow_mut().set_active_flag(false);
                let duration = soundwave7
                    .borrow()
                    .play(Some(lite62), "GravityWellKickoutControl");
                c.borrow_mut()
                    .message(MessageCode::T_KICKOUT_RESTART_TIMER, duration);
            }
        }
        _ => {
            println!("Code not yet implemented: val: {}", code.0);
        }
    }

    Ok(())
}

pub(crate) fn unstuck_ball(p0: &mut TBall, p1: usize) {
    todo!()
}

pub(crate) fn cheat_bump_rank() {
    todo!()
}

// pub(crate) fn make_links(
//     table_weak: Option<Weak<RefCell<TPinballTable>>>,
//     control_state: &mut ControlState,
// ) {
//     control_state.table_g = table_weak;
//
//     for score_component in control_state.score_components {
//         let linked_comp = make_component_link(&score_component.tag);
//         if let Some(lc) = linked_comp.as_mut() {
//             lc.control = &score_component.control;
//         }
//     }
// }
//
// fn make_component_link(base_tag: &ComponentTagBase) -> Option<TPinballComponent> {
//     todo!()
// }
