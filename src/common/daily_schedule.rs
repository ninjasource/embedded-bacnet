use core::fmt::Debug;

use crate::common::{
    error::Error,
    helper::{encode_closing_tag, encode_opening_tag, get_tagged_body},
    io::{Reader, Writer},
    time_value::TimeValue,
};

#[cfg(feature = "alloc")]
use {crate::common::spooky::Phantom, alloc::vec::Vec};

// note that Debug is implemented manually here because of the reader in time value iter
#[cfg(not(feature = "alloc"))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WeeklySchedule<'a> {
    pub monday: TimeValueList<'a>,
    pub tuesday: TimeValueList<'a>,
    pub wednesday: TimeValueList<'a>,
    pub thursday: TimeValueList<'a>,
    pub friday: TimeValueList<'a>,
    pub saturday: TimeValueList<'a>,
    pub sunday: TimeValueList<'a>,
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WeeklySchedule<'a> {
    pub monday: Vec<TimeValue>,
    pub tuesday: Vec<TimeValue>,
    pub wednesday: Vec<TimeValue>,
    pub thursday: Vec<TimeValue>,
    pub friday: Vec<TimeValue>,
    pub saturday: Vec<TimeValue>,
    pub sunday: Vec<TimeValue>,
    _phantom: &'a Phantom,
}

#[cfg(feature = "alloc")]
impl<'a> WeeklySchedule<'a> {
    pub fn new(
        monday: Vec<TimeValue>,
        tuesday: Vec<TimeValue>,
        wednesday: Vec<TimeValue>,
        thursday: Vec<TimeValue>,
        friday: Vec<TimeValue>,
        saturday: Vec<TimeValue>,
        sunday: Vec<TimeValue>,
    ) -> Self {
        use crate::common::spooky::PHANTOM;

        Self {
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            sunday,
            _phantom: &PHANTOM,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        encode_day(writer, self.monday.iter());
        encode_day(writer, self.tuesday.iter());
        encode_day(writer, self.wednesday.iter());
        encode_day(writer, self.thursday.iter());
        encode_day(writer, self.friday.iter());
        encode_day(writer, self.saturday.iter());
        encode_day(writer, self.sunday.iter());
    }

    // due to the fact that WeeklySchedule contains an arbitrary number of TimeValue pairs we need to return an iterator
    // because we cannot use an allocator
    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let monday = Self::decode_day(reader, buf)?;
        let tuesday = Self::decode_day(reader, buf)?;
        let wednesday = Self::decode_day(reader, buf)?;
        let thursday = Self::decode_day(reader, buf)?;
        let friday = Self::decode_day(reader, buf)?;
        let saturday = Self::decode_day(reader, buf)?;
        let sunday = Self::decode_day(reader, buf)?;

        Ok(Self::new(
            monday, tuesday, wednesday, thursday, friday, saturday, sunday,
        ))
    }

    fn decode_day(reader: &mut Reader, buf: &[u8]) -> Result<Vec<TimeValue>, Error> {
        let (body_buf, _tag_num) = get_tagged_body(reader, buf)?;
        let mut inner_reader = Reader::new_with_len(body_buf.len());
        let mut time_values = Vec::new();
        while !inner_reader.eof() {
            let time_value = TimeValue::decode(&mut inner_reader, body_buf)?;
            time_values.push(time_value);
        }
        Ok(time_values)
    }
}

#[cfg(not(feature = "alloc"))]
impl<'a> WeeklySchedule<'a> {
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
            monday: TimeValueList::new(monday),
            tuesday: TimeValueList::new(tuesday),
            wednesday: TimeValueList::new(wednesday),
            thursday: TimeValueList::new(thursday),
            friday: TimeValueList::new(friday),
            saturday: TimeValueList::new(saturday),
            sunday: TimeValueList::new(sunday),
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        self.monday.encode(writer);
        self.tuesday.encode(writer);
        self.wednesday.encode(writer);
        self.thursday.encode(writer);
        self.friday.encode(writer);
        self.saturday.encode(writer);
        self.sunday.encode(writer);
    }

    // due to the fact that WeeklySchedule contains an arbitrary number of TimeValue pairs we need to return an iterator
    // because we cannot use an allocator
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let monday = TimeValueList::decode(reader, buf)?;
        let tuesday = TimeValueList::decode(reader, buf)?;
        let wednesday = TimeValueList::decode(reader, buf)?;
        let thursday = TimeValueList::decode(reader, buf)?;
        let friday = TimeValueList::decode(reader, buf)?;
        let saturday = TimeValueList::decode(reader, buf)?;
        let sunday = TimeValueList::decode(reader, buf)?;

        Ok(Self {
            monday,
            tuesday,
            wednesday,
            thursday,
            friday,
            saturday,
            sunday,
        })
    }
}

// note that Debug is not implemented here because if does not add value
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimeValueList<'a> {
    pub time_values: &'a [TimeValue],
    buf: &'a [u8],
}

fn encode_day<'b>(writer: &mut Writer, time_values: impl Iterator<Item = &'b TimeValue>) {
    encode_opening_tag(writer, 0);
    for time_value in time_values {
        time_value.encode(writer)
    }
    encode_closing_tag(writer, 0);
}

impl<'a> TimeValueList<'a> {
    pub fn new(time_values: &'a [TimeValue]) -> Self {
        Self {
            time_values,
            buf: &[],
        }
    }

    pub fn new_from_buf(buf: &'a [u8]) -> Self {
        Self {
            time_values: &[],
            buf,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        encode_day(writer, self.time_values.iter());
    }

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let (body_buf, _tag_num) = get_tagged_body(reader, buf)?;
        Ok(TimeValueList::new_from_buf(body_buf))
    }
}

impl<'a> IntoIterator for &'_ TimeValueList<'a> {
    type Item = Result<TimeValue, Error>;
    type IntoIter = TimeValueIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TimeValueIter {
            buf: self.buf,
            reader: Reader::new_with_len(self.buf.len()),
        }
    }
}

pub struct TimeValueIter<'a> {
    reader: Reader,
    buf: &'a [u8],
}

impl<'a> Iterator for TimeValueIter<'a> {
    type Item = Result<TimeValue, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            return None;
        }

        Some(TimeValue::decode(&mut self.reader, self.buf))
    }
}
