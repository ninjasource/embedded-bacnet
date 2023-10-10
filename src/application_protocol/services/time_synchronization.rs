use crate::{
    application_protocol::{
        primitives::data_value::{Date, Time},
        unconfirmed::UnconfirmedServiceChoice,
    },
    common::{
        io::Writer,
        tag::{ApplicationTagNumber, Tag, TagNumber},
    },
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimeSynchronization {
    pub date: Date,
    pub time: Time,
}

impl TimeSynchronization {
    pub fn encode(&self, writer: &mut Writer) {
        writer.push(UnconfirmedServiceChoice::TimeSynchronization as u8);

        // date
        let tag = Tag::new(TagNumber::Application(ApplicationTagNumber::Date), 4);
        tag.encode(writer);
        self.date.encode(writer);

        // time
        let tag = Tag::new(TagNumber::Application(ApplicationTagNumber::Time), 4);
        tag.encode(writer);
        self.time.encode(writer);
    }
}
