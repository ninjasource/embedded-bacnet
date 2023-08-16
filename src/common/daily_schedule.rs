use core::fmt::Debug;

use crate::common::helper::{get_reader_for_tag, Writer};

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
        let monday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let tuesday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let wednesday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let thursday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let friday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let saturday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let sunday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);

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

/*
impl WeeklyScheduleIter {
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
                    };s
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
*/

#[derive(Debug)]
pub struct WeeklyScheduleWrite<'a> {
    pub monday: &'a [TimeValue],
    pub tuesday: &'a [TimeValue],
    pub wednesday: &'a [TimeValue],
    pub thursday: &'a [TimeValue],
    pub friday: &'a [TimeValue],
    pub saturday: &'a [TimeValue],
    pub sunday: &'a [TimeValue],
}

#[derive(Debug)]
pub struct WeeklyScheduleNew<T>
where
    T: IntoIterator<Item = TimeValue>,
{
    pub monday: T,
    pub tuesday: T,
    pub wednesday: T,
    pub thursday: T,
    pub friday: T,
    pub saturday: T,
    pub sunday: T,
}

#[derive(Debug)]
pub struct WeeklyScheduleNew1<'a> {
    pub monday: BacList<'a>,
    pub tuesday: BacList<'a>,
    pub wednesday: BacList<'a>,
    pub thursday: BacList<'a>,
    pub friday: BacList<'a>,
    pub saturday: BacList<'a>,
    pub sunday: BacList<'a>,
}

#[derive(Debug)]
pub struct BacList<'a> {
    cursor: usize,
    _buf: &'a [u8],
}

impl<'a> Iterator for BacList<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor == 0 {
            Some(0)
        } else {
            None
        }
    }
}

// dont use generics because they complicate the already-complicated DataLink data structure
impl<T> WeeklyScheduleNew<T>
where
    T: Iterator<Item = TimeValue>,
{
    pub fn encode(&mut self, writer: &mut Writer) {
        Self::encode_day(&mut self.monday, writer);
        Self::encode_day(&mut self.tuesday, writer);
        Self::encode_day(&mut self.wednesday, writer);
        Self::encode_day(&mut self.thursday, writer);
        Self::encode_day(&mut self.friday, writer);
        Self::encode_day(&mut self.saturday, writer);
        Self::encode_day(&mut self.sunday, writer);
    }

    fn encode_day(day: &mut T, writer: &mut Writer) {
        Tag::new(TagNumber::ContextSpecificOpening(0), 0).encode(writer);
        for time_value in day.into_iter() {
            time_value.encode(writer);
        }
        Tag::new(TagNumber::ContextSpecificClosing(0), 0).encode(writer);
    }
}

#[derive(Debug)]
pub struct DayTimeValue {
    // monday is 0
    pub day_of_week: usize,
    pub time_value: TimeValue,
}

impl<'a> WeeklyScheduleWrite<'a> {
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

    // due to the fact that WeeklySchedule contains an arbitrary number of TimeValue pairs we need to return an iterator
    // because we cannot use an allocator
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> WeeklySchedule<'a> {
        let monday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let tuesday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let wednesday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let thursday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let friday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let saturday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);
        let sunday = TimeValueIter::new(get_reader_for_tag(0, reader, buf), buf);

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
