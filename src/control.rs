use crate::context::component_context::ComponentContext;
use crate::maths::Vector3;
use crate::message_code::MessageCode;
use crate::pb;
use crate::state::control_state::CHEAT_LEN;
use crate::state::pinball_state::PinballState;
use crate::t_ball::TBall;
use crate::t_blocker::TBlocker;
use crate::t_collision_component::ICollisionComponent;
use crate::t_edge_segment::IEdgeSegment;
use crate::t_light::TLight;
use crate::t_pinball_component::{IPinballComponent, TPinballComponent};
use crate::t_pinball_table::TPinballTable;
use crate::translations::Msg;
use anyhow::Result;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub fn table_control_handler(
    code: MessageCode,
    component_context: &mut ComponentContext,
) -> Result<()> {
    if code == MessageCode::SET_TILT_LOCK {
        component_context.control_state.table_unlimited_balls = false;
        if let Some(component) = component_context
            .control_state
            .component_state
            .lite_77
            .get()
        {
            component.borrow_mut().message(
                MessageCode::T_LIGHT_FLASHER_START_TIMED,
                0.0f32,
                component_context,
            )?;
        }
    }
    Ok(())
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
    } else if state
        .control_state
        .cheat_buffer
        .borrow_mut()
        .ends_with("bmax".as_ref())
    {
        state.control_state.table_unlimited_balls ^= true;
    } else if state
        .control_state
        .cheat_buffer
        .borrow_mut()
        .ends_with("rmax".as_ref())
    {
        cheat_bump_rank();
    } else if state.pb_game_state.full_tilt_mode
        && state
            .control_state
            .cheat_buffer
            .borrow_mut()
            .ends_with("quote".as_ref())
    {
        // Developer Easter egg type 'cheat' from Full Tilt
        let mut time = 0;
        for quote in QUOTES {
            if let Some(mtb) = state.pb_game_state.mission_text_box.clone() {
                time += 3;
                let mut component_ctx = state.get_component_context();

                mtb.borrow_mut()
                    .display(quote, time as f32, &mut component_ctx, Some(true))?;
            }
        }
        return Ok(());
    } else if state.pb_game_state.full_tilt_mode
        && state
            .control_state
            .cheat_buffer
            .borrow_mut()
            .ends_with("credits".as_ref())
    {
        let mut time = 0;
        for line in CREDITS {
            if let Some(mtb) = state.pb_game_state.mission_text_box.clone() {
                // Manual inst to prevent borrow issues
                let mut component_ctx = state.get_component_context();
                time += 2;
                mtb.borrow_mut()
                    .display(line, time as f32, &mut component_ctx, Some(true))?;
            }
        }
        state.pb_game_state.credits_active = true;
        return Ok(());
    } else if state
        .control_state
        .cheat_buffer
        .borrow_mut()
        .ends_with("easy mode".as_ref())
    {
        state.control_state.easy_mode ^= true;
        if state.control_state.easy_mode {
            let block = state.control_state.component_state.block_1.get();
            let easy_mode = state.control_state.easy_mode;
            let light = state.control_state.component_state.lite_1.get();
            let mut component_ctx = state.get_component_context();
            drain_ball_blocker_control(
                MessageCode::T_BLOCKER_ENABLE,
                block,
                easy_mode,
                light,
                &mut component_ctx,
            )?;
        }
    }

    Ok(())

    // TODO: todo!()
}

fn drain_ball_blocker_control(
    code: MessageCode,
    block: Option<Rc<RefCell<TBlocker>>>,
    easy_mode: bool,
    light: Option<Rc<RefCell<TLight>>>,
    component_context: &mut ComponentContext,
) -> Result<()> {
    // The original casts caller to TBlocker and assigns it to block,
    // but it doesn't use caller as anything else
    match code {
        MessageCode::T_BLOCKER_ENABLE => {
            if let Some(block) = block.as_ref()
                && let Some(lite1) = light.as_ref()
            {
                block.borrow_mut().base.message_field = MessageCode(1);
                let blocker_duration = if !easy_mode {
                    block.borrow().initial_duration as f32
                } else {
                    -1.0f32
                };
                block.borrow_mut().message(
                    MessageCode::T_BLOCKER_ENABLE,
                    blocker_duration,
                    component_context,
                )?;
                lite1.borrow_mut().message(
                    MessageCode::T_LIGHT_TURN_ON_TIMED,
                    blocker_duration,
                    component_context,
                )?;
            }
        }
        MessageCode::CONTROL_TIMER_EXPIRED => {
            if let Some(block) = block.as_ref()
                && let Some(lite1) = light.as_ref()
            {
                if block.borrow().base.message_field == MessageCode(1) {
                    block.borrow_mut().base.message_field = MessageCode(2);
                    let blocker_duration = block.borrow().extended_duration as f32;
                    block.borrow_mut().message(
                        MessageCode::T_BLOCKER_RESTART_TIMEOUT,
                        blocker_duration,
                        component_context,
                    )?;
                    lite1.borrow_mut().message(
                        MessageCode::T_LIGHT_FLASHER_START_TIMED,
                        blocker_duration,
                        component_context,
                    )?;
                } else {
                    block.borrow_mut().base.message_field = MessageCode(0);
                    block.borrow_mut().message(
                        MessageCode::T_BLOCKER_DISABLE,
                        0.0f32,
                        component_context,
                    )?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn table_add_extra_ball(count: f32, state: &mut PinballState) -> Result<()> {
    if let Some(wave) = state.control_state.component_state.soundwave_28.get() {
        wave.borrow().play(None, "table_add_extra_ball");
    }
    if let Some(itb) = state.control_state.component_state.info_text_box.get() {
        let rc_string = pb::get_rc_string(Msg::STRING110)?;
        let mut component_ctx = ComponentContext::from_state(state);
        itb.borrow_mut()
            .display(rc_string, count, &mut component_ctx, None)?;
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
                && let Some(soundwave7) = state.control_state.component_state.soundwave_7.get()
            {
                let scoring = c.borrow().get_scoring(0);
                let added_score = t
                    .borrow_mut()
                    .add_score(scoring, state.pb_game_state.full_tilt_mode)?;

                let rc_string = pb::get_rc_string(Msg::STRING182)?
                    .replace("%ld", added_score.to_string().as_str());

                let mut component_ctx = ComponentContext::from_state(state);
                tb.borrow_mut()
                    .display(&rc_string, 2.0, &mut component_ctx, None)?;

                lite62.borrow_mut().message(
                    MessageCode::T_LIGHT_RESET_AND_TURN_OFF,
                    0.0f32,
                    &mut component_ctx,
                )?;
                c.borrow_mut().set_active_flag(false);
                let duration = soundwave7
                    .borrow()
                    .play(Some(lite62), "GravityWellKickoutControl");
                c.borrow_mut().message(
                    MessageCode::T_KICKOUT_RESTART_TIMER,
                    duration,
                    &mut component_ctx,
                )?;
            }
        }
        _ => {
            println!("Code not yet implemented: val: {}", code.0);
        }
    }

    Ok(())
}

pub(crate) fn cheat_bump_rank() {
    todo!()
}

pub(crate) fn handler(p0: MessageCode, p1: &mut TBlocker) {
    todo!()
}

pub(crate) fn unstuck_ball(
    ball: &mut RefMut<TBall>,
    _dt: usize,
    mut table_opt: Option<Rc<RefCell<TPinballTable>>>,
    comp_ctx: &mut ComponentContext,
) -> Result<()> {
    if let Some(flip1) = comp_ctx.control_state.component_state.flip_1.get()
        && let Some(flip2) = comp_ctx.control_state.component_state.flip_2.get()
        && let Some(plunger) = comp_ctx.control_state.component_state.plunger.get()
        && let Some(table) = table_opt.as_ref()
    {
        let col_offset = table.borrow().collision_comp_offset;

        if !check_ball_in_control_bounds(ball, &flip1, col_offset)
            && !check_ball_in_control_bounds(ball, &flip2, col_offset)
            && !check_ball_in_control_bounds(ball, &plunger, col_offset)
        {
            if ball.stuck_count <= 20 {
                let throw_dir = Vector3::new(0.0, -1.0, 0.0);
                ball.throw_ball(&throw_dir, 90.0, 1.0, 0.0);
            } else {
                ball.disable();
                if let Some(table) = table_opt.as_mut() {
                    table.borrow_mut().multiball_count -= 1;
                    plunger.borrow_mut().message(
                        MessageCode::PLUNGER_RELAUNCH_BALL,
                        0.0,
                        comp_ctx,
                    )?;
                }
            }
        }
    }
    Ok(())
}

pub fn check_ball_in_control_bounds<T: ICollisionComponent + ?Sized>(
    ball: &mut RefMut<TBall>,
    cmp: &Rc<RefCell<T>>,
    collision_comp_offset: f32,
) -> bool {
    let offset = collision_comp_offset / 2.0f32;
    ball.active_flag().get()
        && ball.position.x >= cmp.borrow().get_AABB().unwrap().x_min - offset
        && ball.position.x <= cmp.borrow().get_AABB().unwrap().x_max + offset
        && ball.position.y >= cmp.borrow().get_AABB().unwrap().y_min - offset
        && ball.position.y <= cmp.borrow().get_AABB().unwrap().y_max + offset
}
