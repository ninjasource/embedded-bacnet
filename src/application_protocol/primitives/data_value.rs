use core::{fmt::Display, str::from_utf8};

use crate::common::{
    daily_schedule::WeeklySchedule,
    error::Error,
    helper::{decode_unsigned, encode_application_enumerated},
    io::{Reader, Writer},
    object_id::{ObjectId, ObjectType},
    property_id::PropertyId,
    spec::{
        Binary, EngineeringUnits, EventState, LogBufferResult, LoggingType, NotifyType, Status,
    },
    tag::{ApplicationTagNumber, Tag, TagNumber},
};

#[cfg(feature = "alloc")]
use {
    crate::common::spooky::Phantom,
    alloc::{string::String, vec::Vec},
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ApplicationDataValue<'a> {
    Boolean(bool),
    Real(f32),
    Double(f64),
    Date(Date),
    Time(Time),
    ObjectId(ObjectId),
    CharacterString(CharacterString<'a>),
    Enumerated(Enumerated),
    BitString(BitString<'a>),
    UnsignedInt(u32),
    WeeklySchedule(WeeklySchedule<'a>),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ApplicationDataValueWrite<'a> {
    Boolean(bool),
    Enumerated(Enumerated),
    Real(f32),
    WeeklySchedule(WeeklySchedule<'a>),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Enumerated {
    Units(EngineeringUnits),
    Binary(Binary),
    ObjectType(ObjectType),
    EventState(EventState),
    NotifyType(NotifyType),
    LoggingType(LoggingType),
    Unknown(u32),
}

impl Enumerated {
    pub fn encode(&self, writer: &mut Writer) {
        let value = match self {
            Self::Units(x) => x.clone() as u32,
            Self::Binary(x) => x.clone() as u32,
            Self::ObjectType(x) => *x as u32,
            Self::EventState(x) => x.clone() as u32,
            Self::NotifyType(x) => x.clone() as u32,
            Self::LoggingType(x) => x.clone() as u32,
            Self::Unknown(x) => *x,
        };
        encode_application_enumerated(writer, value);
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub wday: u8, // 1 (Monday) to 7 (Sunday)
}

impl Date {
    pub const LEN: u32 = 4; // 4 bytes

    //  year = years since 1900, wildcard=1900+255
    //  month 1=Jan
    //  day = day of month
    //  wday 1=Monday...7=Sunday
    pub fn decode_from_tag(tag: &Tag) -> Self {
        let value = tag.value;
        let value = value.to_be_bytes();
        Self::decode_inner(value)
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let value = reader.read_bytes(buf)?;
        Ok(Self::decode_inner(value))
    }

    fn decode_inner(value: [u8; 4]) -> Self {
        let year = value[0] as u16 + 1900;
        let month = value[1];
        let day = value[2];
        let wday = value[3];
        Self {
            year,
            month,
            day,
            wday,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        let year = (self.year - 1900) as u8;
        writer.push(year);
        writer.push(self.month);
        writer.push(self.day);
        writer.push(self.wday);
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub hundredths: u8,
}

impl Time {
    pub const LEN: u32 = 4; // 4 bytes

    // assuming that this comes from a Time tag
    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let hour = reader.read_byte(buf)?;
        let minute = reader.read_byte(buf)?;
        let second = reader.read_byte(buf)?;
        let hundredths = reader.read_byte(buf)?;
        Ok(Time {
            hour,
            minute,
            second,
            hundredths,
        })
    }

    pub fn encode(&self, writer: &mut Writer) {
        writer.push(self.hour);
        writer.push(self.minute);
        writer.push(self.second);
        writer.push(self.hundredths);
    }
}

#[cfg(not(feature = "alloc"))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharacterString<'a> {
    pub inner: &'a str,
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CharacterString<'a> {
    pub inner: String,
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    _phantom: &'a Phantom,
}

impl<'a> Display for ApplicationDataValue<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ApplicationDataValue::Real(x) => write!(f, "{}", x),
            ApplicationDataValue::Double(x) => write!(f, "{}", x),
            ApplicationDataValue::CharacterString(x) => write!(f, "{}", &x.inner),
            ApplicationDataValue::Boolean(x) => write!(f, "{}", x),
            x => write!(f, "{:?}", x),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BitString<'a> {
    Status(Status),
    LogBufferResult(LogBufferResult),
    Custom(CustomBitStream<'a>),
}

#[cfg(feature = "defmt")]
impl<'a> defmt::Format for BitString<'a> {
    fn format(&self, _fmt: defmt::Formatter) {
        // do nothing for now because it is too complicated due to StatusFlags
    }
}

#[cfg(not(feature = "alloc"))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CustomBitStream<'a> {
    pub unused_bits: u8,
    pub bits: &'a [u8],
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CustomBitStream<'a> {
    pub unused_bits: u8,
    pub bits: Vec<u8>,
    _phantom: &'a Phantom,
}

impl<'a> CustomBitStream<'a> {
    #[cfg(not(feature = "alloc"))]
    pub fn new(unused_bits: u8, bits: &'a [u8]) -> Self {
        Self { unused_bits, bits }
    }

    #[cfg(feature = "alloc")]
    pub fn new(unused_bits: u8, bits: &'_ [u8]) -> Self {
        use crate::common::spooky::PHANTOM;

        Self {
            unused_bits,
            bits: bits.into(),
            _phantom: &PHANTOM,
        }
    }
}

impl<'a> BitString<'a> {
    pub fn encode_application(&self, writer: &mut Writer) {
        match self {
            Self::Status(x) => {
                Tag::new(TagNumber::Application(ApplicationTagNumber::BitString), 2).encode(writer);
                writer.push(0); // no unused bits
                writer.push(x.inner);
            }
            Self::LogBufferResult(x) => {
                Tag::new(TagNumber::Application(ApplicationTagNumber::BitString), 2).encode(writer);
                writer.push(0); // no unused bits
                writer.push(x.inner);
            }
            Self::Custom(x) => {
                Tag::new(
                    TagNumber::Application(ApplicationTagNumber::BitString),
                    x.bits.len() as u32 + 1,
                )
                .encode(writer);
                writer.push(0); // no unused bits
                writer.extend_from_slice(&x.bits);
            }
        }
    }

    pub fn encode_context(&self, tag_num: u8, writer: &mut Writer) {
        match self {
            Self::Status(x) => {
                Tag::new(TagNumber::ContextSpecific(tag_num), 2).encode(writer);
                writer.push(0); // no unused bits
                writer.push(x.inner);
            }
            Self::LogBufferResult(x) => {
                Tag::new(TagNumber::ContextSpecific(tag_num), 2).encode(writer);
                writer.push(0); // no unused bits
                writer.push(x.inner);
            }
            Self::Custom(x) => {
                Tag::new(TagNumber::ContextSpecific(tag_num), x.bits.len() as u32 + 1)
                    .encode(writer);
                writer.push(0); // no unused bits
                writer.extend_from_slice(&x.bits);
            }
        }
    }

    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(
        property_id: &PropertyId,
        len: u32,
        reader: &mut Reader,
        buf: &'a [u8],
    ) -> Result<Self, Error> {
        let unused_bits = reader.read_byte(buf)?;
        match property_id {
            PropertyId::PropStatusFlags => {
                let status_flags = Status::new(reader.read_byte(buf)?);
                Ok(Self::Status(status_flags))
            }
            PropertyId::PropLogBuffer => {
                let flags = LogBufferResult::new(reader.read_byte(buf)?);
                Ok(Self::LogBufferResult(flags))
            }
            _ => {
                let len = (len - 1) as usize; // we have already read a byte
                let bits = reader.read_slice(len, buf)?;
                Ok(Self::Custom(CustomBitStream::new(unused_bits, bits)))
            }
        }
    }
}

impl<'a> CharacterString<'a> {
    #[cfg(not(feature = "alloc"))]
    pub fn new(inner: &'a str) -> Self {
        Self { inner }
    }

    #[cfg(feature = "alloc")]
    pub fn new(inner: &str) -> Self {
        use crate::common::spooky::PHANTOM;

        Self {
            inner: inner.into(),
            _phantom: &PHANTOM,
        }
    }

    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(len: u32, reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let character_set = reader.read_byte(buf)?;
        if character_set != 0 {
            unimplemented!("non-utf8 characterset not supported")
        }
        let slice = reader.read_slice(len as usize - 1, buf)?;
        let inner = from_utf8(slice).map_err(|_| {
            Error::InvalidValue("CharacterString bytes are not a valid utf8 string")
        })?;

        Ok(CharacterString::new(inner))
    }
}

impl<'a> ApplicationDataValueWrite<'a> {
    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(
        object_id: &ObjectId,
        property_id: &PropertyId,
        reader: &mut Reader,
        buf: &'a [u8],
    ) -> Result<Self, Error> {
        match property_id {
            PropertyId::PropWeeklySchedule => {
                let weekly_schedule = WeeklySchedule::decode(reader, buf)?;
                Ok(Self::WeeklySchedule(weekly_schedule))
            }
            _ => {
                let tag = Tag::decode(reader, buf)?;
                match tag.number {
                    TagNumber::Application(ApplicationTagNumber::Boolean) => {
                        Ok(Self::Boolean(tag.value > 0))
                    }
                    TagNumber::Application(ApplicationTagNumber::Real) => {
                        if tag.value != 4 {
                            return Err(Error::Length((
                                "real tag should have length of 4",
                                tag.value,
                            )));
                        }
                        let bytes = reader.read_bytes(buf)?;
                        Ok(Self::Real(f32::from_be_bytes(bytes)))
                    }
                    TagNumber::Application(ApplicationTagNumber::Enumerated) => {
                        let value = decode_enumerated(object_id, property_id, &tag, reader, buf)?;
                        Ok(Self::Enumerated(value))
                    }
                    tag_number => Err(Error::TagNotSupported((
                        "ApplicationDataValueWrite decode",
                        tag_number,
                    ))),
                }
            }
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        match self {
            Self::Boolean(x) => {
                let len = 1;
                let tag = Tag::new(TagNumber::Application(ApplicationTagNumber::Boolean), len);
                tag.encode(writer);
                let value = if *x { 1_u8 } else { 0_u8 };
                writer.push(value)
            }
            Self::Real(x) => {
                let len = 4;
                let tag = Tag::new(TagNumber::Application(ApplicationTagNumber::Real), len);
                tag.encode(writer);
                writer.extend_from_slice(&f32::to_be_bytes(*x))
            }
            Self::Enumerated(x) => {
                x.encode(writer);
            }
            Self::WeeklySchedule(x) => x.encode(writer),
        }
    }
}

impl<'a> ApplicationDataValue<'a> {
    pub fn encode(&self, writer: &mut Writer) {
        match self {
            ApplicationDataValue::Boolean(x) => Tag::new(
                TagNumber::Application(ApplicationTagNumber::Boolean),
                if *x { 1 } else { 0 },
            )
            .encode(writer),
            ApplicationDataValue::Real(x) => {
                Tag::new(TagNumber::Application(ApplicationTagNumber::Real), 4).encode(writer);
                writer.extend_from_slice(&x.to_be_bytes());
            }
            ApplicationDataValue::Date(x) => {
                Tag::new(
                    TagNumber::Application(ApplicationTagNumber::Date),
                    Date::LEN,
                )
                .encode(writer);
                x.encode(writer);
            }
            ApplicationDataValue::Time(x) => {
                Tag::new(
                    TagNumber::Application(ApplicationTagNumber::Time),
                    Time::LEN,
                )
                .encode(writer);
                x.encode(writer);
            }
            ApplicationDataValue::ObjectId(x) => {
                Tag::new(
                    TagNumber::Application(ApplicationTagNumber::ObjectId),
                    ObjectId::LEN,
                )
                .encode(writer);
                x.encode(writer);
            }
            ApplicationDataValue::CharacterString(x) => {
                let utf8_encoded = x.inner.as_bytes(); // strings in rust are utf8 encoded already
                Tag::new(
                    TagNumber::Application(ApplicationTagNumber::CharacterString),
                    utf8_encoded.len() as u32 + 1, // keep space for encoding byte
                )
                .encode(writer);
                writer.push(0); // utf8 encoding
                writer.extend_from_slice(utf8_encoded);
            }
            ApplicationDataValue::Enumerated(x) => {
                x.encode(writer);
            }
            ApplicationDataValue::BitString(x) => {
                x.encode_application(writer);
            }
            ApplicationDataValue::UnsignedInt(x) => {
                Tag::new(TagNumber::Application(ApplicationTagNumber::UnsignedInt), 4)
                    .encode(writer);
                writer.extend_from_slice(&x.to_be_bytes());
            }
            ApplicationDataValue::WeeklySchedule(x) => {
                // no application tag required for weekly schedule
                x.encode(writer);
            }

            x => todo!("{:?}", x),
        };
    }

    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(
        tag: &Tag,
        object_id: &ObjectId,
        property_id: &PropertyId,
        reader: &mut Reader,
        buf: &'a [u8],
    ) -> Result<Self, Error> {
        let tag_num = match &tag.number {
            TagNumber::Application(x) => x,
            unknown => {
                return Err(Error::TagNotSupported((
                    "Expected Application tag",
                    unknown.clone(),
                )))
            }
        };

        match tag_num {
            ApplicationTagNumber::Real => {
                if tag.value != 4 {
                    return Err(Error::Length((
                        "real tag should have length of 4",
                        tag.value,
                    )));
                }
                Ok(ApplicationDataValue::Real(f32::from_be_bytes(
                    reader.read_bytes(buf)?,
                )))
            }
            ApplicationTagNumber::ObjectId => {
                let object_id = ObjectId::decode(tag.value, reader, buf)?;
                Ok(ApplicationDataValue::ObjectId(object_id))
            }
            ApplicationTagNumber::CharacterString => {
                let text = CharacterString::decode(tag.value, reader, buf)?;
                Ok(ApplicationDataValue::CharacterString(text))
            }
            ApplicationTagNumber::Enumerated => {
                let value = decode_enumerated(object_id, property_id, tag, reader, buf)?;
                Ok(ApplicationDataValue::Enumerated(value))
            }
            ApplicationTagNumber::BitString => {
                let bit_string = BitString::decode(property_id, tag.value, reader, buf)?;
                Ok(ApplicationDataValue::BitString(bit_string))
            }
            ApplicationTagNumber::Boolean => {
                let value = tag.value > 0;
                Ok(ApplicationDataValue::Boolean(value))
            }
            ApplicationTagNumber::UnsignedInt => {
                let value = decode_unsigned(tag.value, reader, buf)? as u32;
                Ok(ApplicationDataValue::UnsignedInt(value))
            }
            ApplicationTagNumber::Time => {
                if tag.value != 4 {
                    return Err(Error::Length((
                        "time tag should have length of 4",
                        tag.value,
                    )));
                }
                let time = Time::decode(reader, buf)?;
                Ok(ApplicationDataValue::Time(time))
            }
            ApplicationTagNumber::Date => {
                // let date = Date::decode_from_tag(&tag);
                let date = Date::decode(reader, buf)?;
                Ok(ApplicationDataValue::Date(date))
            }

            x => Err(Error::TagNotSupported((
                "ApplicationDataValue decode",
                TagNumber::Application(x.clone()),
            ))),
        }
    }
}

fn decode_enumerated(
    object_id: &ObjectId,
    property_id: &PropertyId,
    tag: &Tag,
    reader: &mut Reader,
    buf: &[u8],
) -> Result<Enumerated, Error> {
    let value = decode_unsigned(tag.value, reader, buf)? as u32;
    match property_id {
        PropertyId::PropUnits => {
            let units = value
                .try_into()
                .map_err(|x| Error::InvalidVariant(("EngineeringUnits", x)))?;
            Ok(Enumerated::Units(units))
        }
        PropertyId::PropPresentValue => match object_id.object_type {
            ObjectType::ObjectBinaryInput
            | ObjectType::ObjectBinaryOutput
            | ObjectType::ObjectBinaryValue => {
                let binary = value
                    .try_into()
                    .map_err(|x| Error::InvalidVariant(("Binary", x)))?;
                Ok(Enumerated::Binary(binary))
            }
            _ => Ok(Enumerated::Unknown(value)),
        },
        PropertyId::PropObjectType => {
            let object_type = ObjectType::try_from(value)
                .map_err(|x| Error::InvalidVariant(("ObjectType", x)))?;
            Ok(Enumerated::ObjectType(object_type))
        }
        PropertyId::PropEventState => {
            let event_state = EventState::try_from(value)
                .map_err(|x| Error::InvalidVariant(("EventState", x)))?;
            Ok(Enumerated::EventState(event_state))
        }
        PropertyId::PropNotifyType => {
            let notify_type = NotifyType::try_from(value)
                .map_err(|x| Error::InvalidVariant(("NotifyType", x)))?;
            Ok(Enumerated::NotifyType(notify_type))
        }
        PropertyId::PropLoggingType => {
            let logging_type = LoggingType::try_from(value)
                .map_err(|x| Error::InvalidVariant(("LoggingType", x)))?;
            Ok(Enumerated::LoggingType(logging_type))
        }

        _ => Ok(Enumerated::Unknown(value)),
    }
}
