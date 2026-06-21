use crate::errors::PbError;
use crate::message_code::MessageCode;

pub struct TPlunger;

impl TPlunger {
    pub(crate) fn message(&self, code: MessageCode, x: f32) -> anyhow::Result<(), PbError> {
        todo!()
    }
}
