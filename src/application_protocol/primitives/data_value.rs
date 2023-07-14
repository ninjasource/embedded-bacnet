use core::{fmt::Display, str::from_utf8};

use alloc::vec::Vec;
use flagset::{FlagSet, Flags};

use crate::common::{
    error::Error,
    helper::{decode_unsigned, Reader},
    object_id::{ObjectId, ObjectType},
    property_id::PropertyId,
    spec::{Binary, EngineeringUnits, StatusFlags},
    tag::{ApplicationTagNumber, Tag, TagNumber},
};

#[derive(Debug)]
pub enum ApplicationDataValue<'a> {
    Boolean(bool),
    Real(f32),
    Double(f64),
    Date(Date),
    Time(Time),
    ObjectId(ObjectId),
    CharacterString(CharacterString<'a>),
    Enumerated(Enumerated),
    BitString(BitString),
    UnsignedInt(u32),
}

#[derive(Debug)]
pub enum Enumerated {
    Units(EngineeringUnits),
    Binary(Binary),
    Unknown(u32),
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
pub struct CharacterString<'a> {
    pub inner: &'a str,
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

#[derive(Debug)]
pub enum BitString {
    StatusFlags(FlagSet<StatusFlags>),
    Custom(CustomBitStream),
}

#[derive(Debug)]
pub struct CustomBitStream {
    pub unused_bits: u8,
    pub bits: Vec<u8>,
}

impl BitString {
    pub fn decode(
        property_id: PropertyId,
        len: u32,
        reader: &mut Reader,
        buf: &[u8],
    ) -> Result<Self, Error> {
        let unused_bits = reader.read_byte(buf);
        match property_id {
            PropertyId::PropStatusFlags => {
                let status_flags = Self::decode_byte_flag(reader.read_byte(buf))?;
                Ok(Self::StatusFlags(status_flags))
            }
            _ => {
                let bits = reader.read_slice(len as usize, buf).to_vec();
                Ok(Self::Custom(CustomBitStream { unused_bits, bits }))
            }
        }
    }

    fn decode_byte_flag<T: Flags>(byte: T::Type) -> Result<FlagSet<T>, Error> {
        match FlagSet::new(byte) {
            Ok(x) => Ok(x),
            Err(_) => Err(Error::InvalidValue("invalid flag bitstream")),
        }
    }
}

impl<'a> CharacterString<'a> {
    pub fn decode(len: u32, reader: &mut Reader, buf: &'a [u8]) -> Self {
        let character_set = reader.read_byte(buf);
        if character_set != 0 {
            unimplemented!("non-utf8 characterset not supported")
        }
        let slice = reader.read_slice(len as usize - 1, buf);
        CharacterString {
            inner: from_utf8(slice).unwrap(),
        }
    }
}

impl<'a> ApplicationDataValue<'a> {
    pub fn decode(
        tag: &Tag,
        object_id: &ObjectId,
        property_id: &PropertyId,
        reader: &mut Reader,
        buf: &'a [u8],
    ) -> Self {
        let tag_num = match tag.number {
            TagNumber::Application(x) => x,
            TagNumber::ContextSpecific(_) => panic!("application tag number expected"),
        };

        match tag_num {
            ApplicationTagNumber::Real => {
                assert_eq!(tag.value, 4, "read tag should have length of 4");
                ApplicationDataValue::Real(f32::from_be_bytes(reader.read_bytes(buf)))
            }
            ApplicationTagNumber::ObjectId => {
                let object_id = ObjectId::decode(tag.value, reader, buf).unwrap();
                ApplicationDataValue::ObjectId(object_id)
            }
            ApplicationTagNumber::CharacterString => {
                let text = CharacterString::decode(tag.value, reader, buf);
                ApplicationDataValue::CharacterString(text)
            }
            ApplicationTagNumber::Enumerated => {
                let value = decode_unsigned(tag.value, reader, buf) as u32;
                let value = match property_id {
                    PropertyId::PropUnits => {
                        let units = value.try_into().unwrap();
                        Enumerated::Units(units)
                    }
                    PropertyId::PropPresentValue => match object_id.object_type {
                        ObjectType::ObjectBinaryInput
                        | ObjectType::ObjectBinaryOutput
                        | ObjectType::ObjectBinaryValue => {
                            let binary = value.try_into().unwrap();
                            Enumerated::Binary(binary)
                        }
                        _ => Enumerated::Unknown(value),
                    },

                    _ => Enumerated::Unknown(value),
                };
                ApplicationDataValue::Enumerated(value)
            }
            ApplicationTagNumber::BitString => {
                let bit_string = BitString::decode(*property_id, tag.value, reader, buf).unwrap();
                ApplicationDataValue::BitString(bit_string)
            }
            ApplicationTagNumber::Boolean => {
                let value = tag.value > 0;
                ApplicationDataValue::Boolean(value)
            }
            ApplicationTagNumber::UnsignedInt => {
                let value = decode_unsigned(tag.value, reader, buf) as u32;
                ApplicationDataValue::UnsignedInt(value)
            }

            _ => unimplemented!(),
        }
    }
}
