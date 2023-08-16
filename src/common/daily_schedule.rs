use crate::common::helper::Writer;

use super::{
    helper::Reader,
    tag::{ApplicationTagNumber, Tag, TagNumber},
    time_value::TimeValue,
};

#[derive(Debug)]
pub struct WeeklyScheduleReader {
    day_of_week: usize,
}

impl WeeklyScheduleReader {
    pub fn decode_next(&mut self, reader: &mut Reader, buf: &[u8]) -> Option<DayTimeValue> {
        loop {
            let tag = Tag::decode(reader, buf);
            match tag.number {
                TagNumber::ContextSpecificClosing(4) => return None,
                TagNumber::ContextSpecificOpening(0) => {
                    // do nothing
                }
                TagNumber::ContextSpecificClosing(0) => {
                    self.day_of_week += 1;
                }
                TagNumber::Application(ApplicationTagNumber::Time) => {
                    let time_value = TimeValue::decode(&tag, reader, buf);
                    let day_time_value = DayTimeValue {
                        day_of_week: self.day_of_week,
                        time_value,
                    };
                    return Some(day_time_value);
                }
                unexpected => panic!(
                    "unexpected tag when decoding weekly schedule: {:?}",
                    unexpected
                ),
            }
        }
    }
}

#[derive(Debug)]
pub struct WeeklySchedule<'a> {
    pub monday: &'a [TimeValue],
    pub tuesday: &'a [TimeValue],
    pub wednesday: &'a [TimeValue],
    pub thursday: &'a [TimeValue],
    pub friday: &'a [TimeValue],
    pub saturday: &'a [TimeValue],
    pub sunday: &'a [TimeValue],
}

#[derive(Debug)]
pub struct WeeklyScheduleNew<'a, T>
where
    T: IntoIterator<Item = &'a TimeValue>,
{
    pub monday: T,
    pub tuesday: T,
    pub wednesday: T,
    pub thursday: T,
    pub friday: T,
    pub saturday: T,
    pub sunday: T,
}

impl<'a, T> WeeklyScheduleNew<'a, T>
where
    T: Iterator<Item = &'a TimeValue>,
{
    // assuming that day_of_week are in ascending order
    pub fn encode(&mut self, writer: &mut Writer) {
        // let mut cha = (&self.monday).into_iter();

        //  let bloot = cha.next();

        Self::encode_day(&self.monday, writer);
        Self::encode_day(&self.tuesday, writer);
        Self::encode_day(&self.wednesday, writer);
        Self::encode_day(&self.thursday, writer);
        Self::encode_day(&self.friday, writer);
        Self::encode_day(&self.saturday, writer);
        Self::encode_day(&self.sunday, writer);
    }

    fn encode_day(_day: &T, writer: &mut Writer) {
        Tag::new(TagNumber::ContextSpecificOpening(0), 0).encode(writer);
        // for time_value in day.into_iter() {
        //     time_value.encode(writer);
        // }
        Tag::new(TagNumber::ContextSpecificClosing(0), 0).encode(writer);
    }
}

#[derive(Debug)]
pub struct DayTimeValue {
    // monday is 0
    pub day_of_week: usize,
    pub time_value: TimeValue,
}

impl<'a> WeeklySchedule<'a> {
    pub fn new() -> Self {
        Self {
            monday: &[],
            tuesday: &[],
            wednesday: &[],
            thursday: &[],
            friday: &[],
            saturday: &[],
            sunday: &[],
        }
    }

    pub fn decode(&self) -> WeeklyScheduleReader {
        WeeklyScheduleReader { day_of_week: 0 }
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
