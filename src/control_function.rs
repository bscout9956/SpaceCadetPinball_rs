// void control::BumperControl(MessageCode code, TPinballComponent* caller)
// {
//     if (code == MessageCode::ControlCollision)
//     {
//         TableG->AddScore(caller->get_scoring(static_cast<TBumper*>(caller)->BmpIndex));
//     }
// }

use crate::message_code::MessageCode;
use crate::t_pinball_component::IPinballComponent;
use crate::t_pinball_table::TPinballTable;
use anyhow::Result;
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn bumper_control(
    code: MessageCode,
    caller: Rc<RefCell<dyn IPinballComponent>>,
    mut table_opt: Option<Rc<RefCell<TPinballTable>>>,
    full_tilt_mode: bool,
) -> Result<()> {
    let caller_borrow = caller.borrow();
    let t_bumper_opt = caller_borrow.as_tbumper();
    if code == MessageCode::CONTROL_COLLISION
        && let Some(table) = table_opt.as_mut() && let Some(t_bumper) = t_bumper_opt {
            table.borrow_mut().add_score(
                caller
                    .borrow()
                    .get_scoring(t_bumper.bmp_index as u32),
                full_tilt_mode,
            )?;
        }
    Ok(())
}
