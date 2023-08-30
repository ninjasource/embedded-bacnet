use core::fmt::Debug;

use super::{
    helper::get_tagged_body,
    io::{Reader, Writer},
    tag::{ApplicationTagNumber, Tag, TagNumber},
    time_value::TimeValue,
};

//// note that Debug is implemented manually here because of the reader in time value iter
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WeeklySchedule<'a> {
    pub monday: TimeValueIter<'a>,
    pub tuesday: TimeValueIter<'a>,
    pub wednesday: TimeValueIter<'a>,
    pub thursday: TimeValueIter<'a>,
    pub friday: TimeValueIter<'a>,
    pub saturday: TimeValueIter<'a>,
    pub sunday: TimeValueIter<'a>,
}

impl<'a> WeeklySchedule<'a> {
    // due to the fact that WeeklySchedule contains an arbitrary number of TimeValue pairs we need to return an iterator
    // because we cannot use an allocator
    pub fn new(reader: &mut Reader, buf: &'a [u8]) -> Self {
        let monday = TimeValueIter::new(get_tagged_body(reader, buf).0);
        let tuesday = TimeValueIter::new(get_tagged_body(reader, buf).0);
        let wednesday = TimeValueIter::new(get_tagged_body(reader, buf).0);
        let thursday = TimeValueIter::new(get_tagged_body(reader, buf).0);
        let friday = TimeValueIter::new(get_tagged_body(reader, buf).0);
        let saturday = TimeValueIter::new(get_tagged_body(reader, buf).0);
        let sunday = TimeValueIter::new(get_tagged_body(reader, buf).0);

        let schedule = WeeklySchedule {
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            sunday,
        };

        // read closing tag
        // let tag = Tag::decode(reader, buf);
        //  assert_eq!(tag.number, TagNumber::ContextSpecificClosing(4));

        schedule
    }
}

//// note that Debug is not implemented here because if does not add value
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimeValueIter<'a> {
    reader: Reader,
    buf: &'a [u8],
}

impl<'a> TimeValueIter<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            reader: Reader {
                index: 0,
                end: buf.len(),
            },
            buf,
        }
    }
}

/*
#[cfg(feature = "defmt")]
impl<'a> defmt::Format for WeeklySchedule<'a> {
    fn format(&self, _fmt: defmt::Formatter) {
        // do nothing
    }
}

impl<'a> Debug for WeeklySchedule<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WeeklyScheduleIter").finish()
    }
}
*/

impl<'a> Iterator for TimeValueIter<'a> {
    type Item = TimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            return None;
        }

        let tag = Tag::decode(&mut self.reader, &self.buf);
        match tag.number {
            TagNumber::Application(ApplicationTagNumber::Time) => {
                let time_value = TimeValue::decode(&tag, &mut self.reader, &self.buf);
                return Some(time_value);
            }
            unexpected => panic!(
                "unexpected tag when decoding weekly schedule: {:?}",
                unexpected
            ),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WeeklyScheduleWrite<'a> {
    monday: &'a [TimeValue],
    tuesday: &'a [TimeValue],
    wednesday: &'a [TimeValue],
    thursday: &'a [TimeValue],
    friday: &'a [TimeValue],
    saturday: &'a [TimeValue],
    sunday: &'a [TimeValue],
}

impl<'a> WeeklyScheduleWrite<'a> {
    pub fn new(
        monday: &'a [TimeValue],
        tuesday: &'a [TimeValue],
        wednesday: &'a [TimeValue],
        thursday: &'a [TimeValue],
        friday: &'a [TimeValue],
        saturday: &'a [TimeValue],
        sunday: &'a [TimeValue],
    ) -> Self {
        Self {
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            sunday,
        }
    }

    // assuming that day_of_week are in ascending order
    pub fn encode(&self, writer: &mut Writer) {
        Self::encode_day(self.monday, writer);
        Self::encode_day(self.tuesday, writer);
        Self::encode_day(self.wednesday, writer);
        Self::encode_day(self.thursday, writer);
        Self::encode_day(self.friday, writer);
        Self::encode_day(self.saturday, writer);
        Self::encode_day(self.sunday, writer);
    }

    fn encode_day(day: &[TimeValue], writer: &mut Writer) {
        Tag::new(TagNumber::ContextSpecificOpening(0), 0).encode(writer);
        for time_value in day {
            time_value.encode(writer);
        }
        Tag::new(TagNumber::ContextSpecificClosing(0), 0).encode(writer);
    }
}
