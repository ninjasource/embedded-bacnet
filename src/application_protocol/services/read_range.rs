use crate::{
    application_protocol::primitives::data_value::{BitString, Date, Time},
    common::{
        helper::{
            decode_context_enumerated, decode_unsigned, encode_application_signed,
            encode_application_unsigned, encode_closing_tag, encode_context_enumerated,
            encode_context_object_id, encode_context_unsigned, encode_opening_tag, get_tagged_body,
        },
        io::{Reader, Writer},
        object_id::ObjectId,
        property_id::PropertyId,
        spec::BACNET_ARRAY_ALL,
        tag::{ApplicationTagNumber, Tag, TagNumber},
    },
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRange {
    pub object_id: ObjectId,     // e.g ObjectTrendLog
    pub property_id: PropertyId, // e.g. PropLogBuffer
    pub array_index: u32,        // use BACNET_ARRAY_ALL for all
    pub request_type: ReadRangeRequestType,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadRangeRequestType {
    ByPosition(ReadRangeByPosition),
    BySequence(ReadRangeBySequence),
    ByTime(ReadRangeByTime),
    All,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeByPosition {
    pub index: u32,
    pub count: u32,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeBySequence {
    pub sequence_num: u32,
    pub count: u32,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeByTime {
    pub date: Date,
    pub time: Time,
    pub count: u32,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeAck<'a> {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
    pub array_index: u32,
    pub result_flags: BitString<'a>,
    pub item_count: usize,
    pub item_data: ReadRangeItems<'a>,
}

impl<'a> ReadRangeAck<'a> {
    const OBJECT_ID_TAG: u8 = 0;
    const PROPERTY_ID_TAG: u8 = 1;
    const ARRAY_INDEX_TAG: u8 = 2;
    const RESULT_FLAGS_TAG: u8 = 3;
    const ITEM_COUNT_TAG: u8 = 4;
    const ITEM_DATA_TAG: u8 = 5;

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Self {
        // object_id
        let tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(Self::OBJECT_ID_TAG),
            "invalid object id tag"
        );
        let object_id = ObjectId::decode(tag.value, reader, buf).unwrap();

        // property_id
        let (tag, property_id) = decode_context_enumerated(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(Self::PROPERTY_ID_TAG),
            "invalid property id tag"
        );

        // array_index
        let mut tag = Tag::decode(reader, buf);
        let mut array_index = BACNET_ARRAY_ALL;
        if let TagNumber::ContextSpecific(Self::ARRAY_INDEX_TAG) = tag.number {
            array_index = decode_unsigned(tag.value, reader, buf) as u32;

            // read another tag
            tag = Tag::decode(reader, buf);
        }

        // result flags
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(Self::RESULT_FLAGS_TAG),
            "invalid result flags tag"
        );
        let result_flags = BitString::decode(property_id, tag.value, reader, buf).unwrap();

        // item_count
        let tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(Self::ITEM_COUNT_TAG),
            "invalid item_count tag"
        );
        let item_count = decode_unsigned(tag.value, reader, buf) as usize;

        // item_data
        let buf = if reader.eof() {
            &[]
        } else {
            let (buf, tag_number) = get_tagged_body(reader, buf);
            assert_eq!(tag_number, Self::ITEM_DATA_TAG, "invalid item_data tag");
            buf
        };
        let item_data = ReadRangeItems::new(buf);

        Self {
            object_id,
            property_id,
            array_index,
            result_flags,
            item_count,
            item_data,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeItems<'a> {
    reader: Reader,
    buf: &'a [u8],
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ReadRangeValue {
    Status,
    Bool(bool),
    Real(f32),
    Enum(u32),
    Unsigned(u32),
    Signed(i32),
    Bits,
    Null,
    Error,
    Delta,
    Any,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
enum ReadRangeValueType {
    Status = 0,
    Bool = 1,
    Real = 2,
    Enum = 3,
    Unsigned = 4,
    Signed = 5,
    Bits = 6,
    Null = 7,
    Error = 8,
    Delta = 9,
    Any = 10,
}

impl TryFrom<u8> for ReadRangeValueType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        match value {
            0 => Ok(Self::Status),
            1 => Ok(Self::Bool),
            2 => Ok(Self::Real),
            3 => Ok(Self::Enum),
            4 => Ok(Self::Unsigned),
            5 => Ok(Self::Signed),
            6 => Ok(Self::Bits),
            7 => Ok(Self::Null),
            8 => Ok(Self::Error),
            9 => Ok(Self::Delta),
            10 => Ok(Self::Any),
            unknown => Err(unknown),
        }
    }
}

impl<'a> ReadRangeItems<'a> {
    const DATE_TIME_TAG: u8 = 0;
    const VALUE_TAG: u8 = 1;
    const STATUS_FLAGS_TAG: u8 = 2;

    pub fn new(buf: &'a [u8]) -> Self {
        let reader = Reader {
            index: 0,
            end: buf.len(),
        };

        Self { reader, buf }
    }
}

impl<'a> Iterator for ReadRangeItems<'a> {
    type Item = ReadRangeItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            return None;
        }

        // date and time
        let tag = Tag::decode(&mut self.reader, self.buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecificOpening(Self::DATE_TIME_TAG),
            "invalid date time opening tag"
        );
        let tag = Tag::decode(&mut self.reader, self.buf);
        assert_eq!(
            tag.number,
            TagNumber::Application(ApplicationTagNumber::Date),
            "expected date application tag"
        );
        let date = Date::decode(&mut self.reader, self.buf);
        let tag = Tag::decode(&mut self.reader, self.buf);
        assert_eq!(
            tag.number,
            TagNumber::Application(ApplicationTagNumber::Time),
            "expected time application tag"
        );
        let time = Time::decode(&mut self.reader, self.buf);
        let tag = Tag::decode(&mut self.reader, self.buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecificClosing(Self::DATE_TIME_TAG),
            "invalid date time closing tag"
        );

        // value
        let tag = Tag::decode(&mut self.reader, self.buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecificOpening(Self::VALUE_TAG),
            "invalid value opening tag"
        );
        let tag = Tag::decode(&mut self.reader, self.buf);
        let value_type: ReadRangeValueType = match tag.number {
            TagNumber::ContextSpecific(tag_number) => tag_number.try_into().unwrap(),
            x => panic!("Unexpected tag found when reading value type: {:?}", x),
        };
        let value = match value_type {
            ReadRangeValueType::Real => {
                let value = f32::from_be_bytes(self.reader.read_bytes(self.buf));
                ReadRangeValue::Real(value)
            }
            _x => unimplemented!(),
        };
        let tag = Tag::decode(&mut self.reader, self.buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecificClosing(Self::VALUE_TAG),
            "invalid value closing tag"
        );

        // status flags
        let tag = Tag::decode(&mut self.reader, self.buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(Self::STATUS_FLAGS_TAG),
            "invalid status flags tag"
        );
        let status_flags = BitString::decode(
            PropertyId::PropStatusFlags,
            tag.value,
            &mut self.reader,
            self.buf,
        )
        .unwrap();

        Some(ReadRangeItem {
            date,
            time,
            value,
            status_flags,
        })
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadRangeItem<'a> {
    pub date: Date,
    pub time: Time,
    pub value: ReadRangeValue,
    pub status_flags: BitString<'a>,
}

impl ReadRange {
    const OBJECT_ID_TAG: u8 = 0;
    const PROPERTY_ID_TAG: u8 = 1;
    const ARRAY_INDEX_TAG: u8 = 2;
    const BY_POSITION_TAG: u8 = 3;
    const BY_SEQUENCE_TAG: u8 = 6;
    const BY_TIME_TAG: u8 = 7;

    pub fn new(
        object_id: ObjectId,
        property_id: PropertyId,
        request_type: ReadRangeRequestType,
    ) -> Self {
        Self {
            object_id,
            property_id,
            array_index: BACNET_ARRAY_ALL,
            request_type,
        }
    }

    pub fn decode(_reader: &mut Reader) {
        unimplemented!("handle read_range only required for a server. see ReadRangeAck for client");
    }

    pub fn encode(&self, writer: &mut Writer) {
        // object_id
        encode_context_object_id(writer, Self::OBJECT_ID_TAG, &self.object_id);

        // property_id
        encode_context_enumerated(writer, Self::PROPERTY_ID_TAG, self.property_id);

        // array_index
        if self.array_index != BACNET_ARRAY_ALL {
            encode_context_unsigned(writer, Self::ARRAY_INDEX_TAG, self.array_index);
        }

        match &self.request_type {
            ReadRangeRequestType::ByPosition(x) => {
                encode_opening_tag(writer, Self::BY_POSITION_TAG);
                encode_application_unsigned(writer, x.index as u64);
                encode_application_signed(writer, x.count as i32);
                encode_closing_tag(writer, Self::BY_POSITION_TAG);
            }
            ReadRangeRequestType::BySequence(x) => {
                encode_opening_tag(writer, Self::BY_SEQUENCE_TAG);
                encode_application_unsigned(writer, x.sequence_num as u64);
                encode_application_signed(writer, x.count as i32);
                encode_closing_tag(writer, Self::BY_SEQUENCE_TAG);
            }
            ReadRangeRequestType::ByTime(x) => {
                encode_opening_tag(writer, Self::BY_TIME_TAG);
                x.date.encode(writer);
                x.time.encode(writer);
                encode_application_signed(writer, x.count as i32);
                encode_closing_tag(writer, Self::BY_TIME_TAG);
            }
            ReadRangeRequestType::All => {
                // do nothing
            }
        }
    }
}
