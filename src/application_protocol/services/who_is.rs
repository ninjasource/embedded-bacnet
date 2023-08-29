use crate::{
    application_protocol::unconfirmed::UnconfirmedServiceChoice,
    common::helper::{Reader, Writer},
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WhoIs {}

impl WhoIs {
    pub fn encode(&self, writer: &mut Writer) {
        writer.push(UnconfirmedServiceChoice::WhoIs as u8)
    }

    pub fn decode(_reader: &mut Reader, _buf: &[u8]) -> Self {
        Self {}
    }
}
