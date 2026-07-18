use crate::context::component_context::ComponentContext;
use crate::message_code::MessageCode;
use crate::t_pinball_component::IPinballComponent;
use anyhow::Result;

pub(crate) fn bumper_control(
    code: MessageCode,
    caller: &mut dyn IPinballComponent,
    ctx: &mut ComponentContext,
) -> Result<()> {
    let t_bumper_opt = caller.as_tbumper();
    if code == MessageCode::CONTROL_COLLISION
        && let Some(table) = ctx.main_table.as_ref()
        && let Some(t_bumper) = t_bumper_opt
    {
        table.borrow_mut().add_score(
            caller.get_scoring(t_bumper.bmp_index as u32),
            ctx.full_tilt_mode,
        )?;
    }
    Ok(())
}

pub(crate) fn right_kicker_gate_control(
    code: MessageCode,
    _caller: &mut dyn IPinballComponent,
    ctx: &mut ComponentContext,
) -> Result<()> {
    if code == MessageCode::T_GATE_DISABLE {
        ctx.control_state
            .component_state
            .lite_29
            .get()
            .unwrap()
            .borrow_mut()
            .message(
                &mut { MessageCode::T_LIGHT_FLASHER_START_TIMED_THEN_STAY_ON },
                5.0,
                ctx,
            )?;
        ctx.control_state
            .component_state
            .lite_195
            .get()
            .unwrap()
            .borrow_mut()
            .message(&mut { MessageCode::T_LIGHT_FLASHER_START_TIMED }, 5.0, ctx)?;
    } else if code == MessageCode::T_GATE_ENABLE {
        ctx.control_state
            .component_state
            .lite_29
            .get()
            .unwrap()
            .borrow_mut()
            .message(&mut { MessageCode::T_LIGHT_RESET_AND_TURN_OFF }, 0.0, ctx)?;
        ctx.control_state
            .component_state
            .lite_195
            .get()
            .unwrap()
            .borrow_mut()
            .message(&mut { MessageCode::T_LIGHT_RESET_AND_TURN_OFF }, 0.0, ctx)?;
    }
    Ok(())
}

pub(crate) fn left_kicker_gate_control(
    code: MessageCode,
    _caller: &mut dyn IPinballComponent,
    ctx: &mut ComponentContext,
) -> Result<()> {
    if code == MessageCode::T_GATE_DISABLE {
        ctx.control_state
            .component_state
            .lite_30
            .get()
            .unwrap()
            .borrow_mut()
            .message(
                &mut { MessageCode::T_LIGHT_FLASHER_START_TIMED_THEN_STAY_ON },
                5.0,
                ctx,
            )?;
        ctx.control_state
            .component_state
            .lite_196
            .get()
            .unwrap()
            .borrow_mut()
            .message(&mut { MessageCode::T_LIGHT_FLASHER_START_TIMED }, 5.0, ctx)?;
    } else if code == MessageCode::T_GATE_ENABLE {
        ctx.control_state
            .component_state
            .lite_30
            .get()
            .unwrap()
            .borrow_mut()
            .message(&mut { MessageCode::T_LIGHT_RESET_AND_TURN_OFF }, 0.0, ctx)?;
        ctx.control_state
            .component_state
            .lite_196
            .get()
            .unwrap()
            .borrow_mut()
            .message(&mut { MessageCode::T_LIGHT_RESET_AND_TURN_OFF }, 0.0, ctx)?;
    }
    Ok(())
}
