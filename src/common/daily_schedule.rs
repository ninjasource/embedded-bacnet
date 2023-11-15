use core::fmt::Debug;

use super::{
    helper::{encode_closing_tag, encode_opening_tag, get_tagged_body},
    io::{Reader, Writer},
    tag::{ApplicationTagNumber, Tag, TagNumber},
    time_value::TimeValue,
};

//// note that Debug is implemented manually here because of the reader in time value iter
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

    // due to the fact that WeeklySchedule contains an arbitrary number of TimeValue pairs we need to return an iterator
    // because we cannot use an allocator
    pub fn new_from_buf(reader: &mut Reader, buf: &'a [u8]) -> Self {
        let monday = TimeValueList::new_from_buf(get_tagged_body(reader, buf).0);
        let tuesday = TimeValueList::new_from_buf(get_tagged_body(reader, buf).0);
        let wednesday = TimeValueList::new_from_buf(get_tagged_body(reader, buf).0);
        let thursday = TimeValueList::new_from_buf(get_tagged_body(reader, buf).0);
        let friday = TimeValueList::new_from_buf(get_tagged_body(reader, buf).0);
        let saturday = TimeValueList::new_from_buf(get_tagged_body(reader, buf).0);
        let sunday = TimeValueList::new_from_buf(get_tagged_body(reader, buf).0);

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

    pub fn encode(&self, writer: &mut Writer) {
        self.monday.encode(writer);
        self.tuesday.encode(writer);
        self.wednesday.encode(writer);
        self.thursday.encode(writer);
        self.friday.encode(writer);
        self.saturday.encode(writer);
        self.sunday.encode(writer);
    }
}

//// note that Debug is not implemented here because if does not add value
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TimeValueList<'a> {
    pub time_values: &'a [TimeValue],
    reader: Reader,
    buf: &'a [u8],
}

impl<'a> TimeValueList<'a> {
    pub fn new(time_values: &'a [TimeValue]) -> Self {
        Self {
            time_values,
            reader: Reader::default(),
            buf: &[],
        }
    }

    pub fn new_from_buf(buf: &'a [u8]) -> Self {
        Self {
            time_values: &[],
            reader: Reader {
                index: 0,
                end: buf.len(),
            },
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
}

impl<'a> Iterator for TimeValueList<'a> {
    type Item = TimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            return None;
        }

        let tag = Tag::decode(&mut self.reader, self.buf);
        match tag.number {
            TagNumber::Application(ApplicationTagNumber::Time) => {
                let time_value = TimeValue::decode(&tag, &mut self.reader, self.buf);
                Some(time_value)
            }
            unexpected => panic!(
                "unexpected tag when decoding weekly schedule: {:?}",
                unexpected
            ),
        }
    }
}
