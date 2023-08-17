use core::fmt::Debug;

use crate::common::helper::{get_tag_body, Writer};

use super::{
    helper::Reader,
    tag::{ApplicationTagNumber, Tag, TagNumber},
    time_value::TimeValue,
};

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
        let monday = TimeValueIter::new(get_tag_body(0, reader, buf), buf);
        let tuesday = TimeValueIter::new(get_tag_body(0, reader, buf), buf);
        let wednesday = TimeValueIter::new(get_tag_body(0, reader, buf), buf);
        let thursday = TimeValueIter::new(get_tag_body(0, reader, buf), buf);
        let friday = TimeValueIter::new(get_tag_body(0, reader, buf), buf);
        let saturday = TimeValueIter::new(get_tag_body(0, reader, buf), buf);
        let sunday = TimeValueIter::new(get_tag_body(0, reader, buf), buf);

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
        let tag = Tag::decode(reader, buf);
        assert_eq!(tag.number, TagNumber::ContextSpecificClosing(4));

        schedule
    }
}

pub struct TimeValueIter<'a> {
    reader: Reader,
    buf: &'a [u8],
}

impl<'a> TimeValueIter<'a> {
    pub fn new(reader: Reader, buf: &'a [u8]) -> Self {
        Self { reader, buf }
    }
}

impl<'a> Debug for WeeklySchedule<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WeeklyScheduleIter").finish()
    }
}

impl<'a> Iterator for TimeValueIter<'a> {
    type Item = TimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        let tag = Tag::decode(&mut self.reader, &self.buf);
        match tag.number {
            TagNumber::ContextSpecificClosing(0) => {
                return None;
            }
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
