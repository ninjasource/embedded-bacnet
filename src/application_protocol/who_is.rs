use crate::common::helper::{Buffer, Reader};

use super::application_pdu::UnconfirmedServiceChoice;

#[derive(Debug)]
pub struct WhoIs {}

impl WhoIs {
    pub fn encode(&self, buffer: &mut Buffer) {
        buffer.push(UnconfirmedServiceChoice::WhoIs as u8)
    }

    pub fn decode(_reader: &mut Reader) -> Self {
        Self {}
    }
}
