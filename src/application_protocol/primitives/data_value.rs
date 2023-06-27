use core::{fmt::Display, str::from_utf8};

use alloc::{borrow::ToOwned, string::String};

use crate::common::{
    helper::{decode_unsigned, Reader},
    object_id::ObjectId,
    property_id::PropertyId,
    tag::{ApplicationTagNumber, Tag, TagNumber},
};

#[derive(Debug)]
pub enum ApplicationDataValue {
    Boolean(bool),
    Real(f32),
    Double(f64),
    Date(Date),
    Time(Time),
    ObjectId(ObjectId),
    CharacterString(CharacterString),
    Enumerated(PropertyId),
    BitString(BitString),
    UnsignedInt(u32),
}

#[derive(Debug)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub wday: u8, // 1 (Monday) to 7 (Sunday)
}

#[derive(Debug)]
pub struct Time {
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub hundredths: u8,
}

#[derive(Debug)]
pub struct CharacterString {
    pub inner: String,
}

impl Display for ApplicationDataValue {
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

#[derive(Debug)]
pub struct BitString {}

impl BitString {
    pub fn decode(reader: &mut Reader, len: u32) -> Self {
        // TODO: do something with the data
        let _ = reader.read_slice(len as usize);
        Self {}
    }
}

impl CharacterString {
    pub fn decode(reader: &mut Reader, len: u32) -> Self {
        let character_set = reader.read_byte();
        if character_set != 0 {
            unimplemented!("non-utf8 characterset not supported")
        }
        let slice = reader.read_slice(len as usize - 1);
        CharacterString {
            inner: from_utf8(slice).unwrap().to_owned(),
        }
    }
}

impl ApplicationDataValue {
    pub fn decode(tag: &Tag, reader: &mut Reader) -> Self {
        let tag_num = match tag.number {
            TagNumber::Application(x) => x,
            TagNumber::ContextSpecific(_) => panic!("application tag number expected"),
        };

        match tag_num {
            ApplicationTagNumber::Real => {
                assert_eq!(tag.value, 4, "read tag should have length of 4");
                ApplicationDataValue::Real(f32::from_be_bytes(reader.read_bytes()))
            }
            ApplicationTagNumber::ObjectId => {
                let object_id = ObjectId::decode(reader, tag.value).unwrap();
                ApplicationDataValue::ObjectId(object_id)
            }
            ApplicationTagNumber::CharacterString => {
                let text = CharacterString::decode(reader, tag.value);
                ApplicationDataValue::CharacterString(text)
            }
            ApplicationTagNumber::Enumerated => {
                let property_id: PropertyId = (decode_unsigned(reader, tag.value) as u32).into();
                ApplicationDataValue::Enumerated(property_id)
            }
            ApplicationTagNumber::BitString => {
                let bit_string = BitString::decode(reader, tag.value);
                ApplicationDataValue::BitString(bit_string)
            }
            ApplicationTagNumber::Boolean => {
                let value = tag.value > 0;
                ApplicationDataValue::Boolean(value)
            }
            ApplicationTagNumber::UnsignedInt => {
                let value = decode_unsigned(reader, tag.value) as u32;
                ApplicationDataValue::UnsignedInt(value)
            }

            _ => unimplemented!(),
        }
    }
}
