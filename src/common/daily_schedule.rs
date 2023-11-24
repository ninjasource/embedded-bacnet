use core::fmt::Debug;

use super::{
    error::Error,
    helper::{encode_closing_tag, encode_opening_tag, get_tagged_body},
    io::{Reader, Writer},
    time_value::TimeValue,
};

// note that Debug is implemented manually here because of the reader in time value iter
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
        encode_opening_tag(writer, 0);
        for time_value in self.time_values {
            time_value.encode(writer)
        }
        encode_closing_tag(writer, 0);
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
