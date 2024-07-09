use crate::{
    application_protocol::primitives::data_value::{Enumerated, Time},
    common::{
        error::{Error, Unimplemented},
        helper::decode_unsigned,
        io::{Reader, Writer},
        spec::Binary,
        tag::{ApplicationTagNumber, Tag, TagNumber},
    },
};

// A simplified version of the ApplicationDataValue struct to avoid a recursive structure
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SimpleApplicationDataValue {
    Boolean(bool),
    SignedInt(i32),
    UnsignedInt(u32),
    Real(f32),
    Double(f64),
    Enumerated(Enumerated),
}

impl SimpleApplicationDataValue {
    pub fn tag(&self) -> Tag {
        match self {
            Self::Boolean(_) => Tag::new(TagNumber::Application(ApplicationTagNumber::Boolean), 1),
            Self::SignedInt(_) => {
                Tag::new(TagNumber::Application(ApplicationTagNumber::SignedInt), 4)
            }
            Self::UnsignedInt(_) => {
                Tag::new(TagNumber::Application(ApplicationTagNumber::UnsignedInt), 4)
            }
            Self::Real(_) => Tag::new(TagNumber::Application(ApplicationTagNumber::Real), 4),
            Self::Double(_) => Tag::new(TagNumber::Application(ApplicationTagNumber::Double), 8),
            Self::Enumerated(_) => {
                Tag::new(TagNumber::Application(ApplicationTagNumber::Enumerated), 1)
            }
        }
    }
    pub fn decode(tag: &Tag, reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let tag_num = match &tag.number {
            TagNumber::Application(x) => x,
            unknown => {
                return Err(Error::TagNotSupported((
                    "SimpleApplicationDataValue decode",
                    unknown.clone(),
                )));
            }
        };

        match tag_num {
            ApplicationTagNumber::Boolean => {
                let value = tag.value > 0;
                Ok(SimpleApplicationDataValue::Boolean(value))
            }
            ApplicationTagNumber::UnsignedInt => {
                let value = decode_unsigned(tag.value, reader, buf)? as u32;
                Ok(SimpleApplicationDataValue::UnsignedInt(value))
            }
            ApplicationTagNumber::Real => {
                if tag.value != 4 {
                    return Err(Error::InvalidValue(
                        "real value tag should have length of 4",
                    ));
                }
                Ok(SimpleApplicationDataValue::Real(f32::from_be_bytes(
                    reader.read_bytes(buf)?,
                )))
            }
            ApplicationTagNumber::Enumerated => {
                let value = decode_unsigned(tag.value, reader, buf)? as u32;
                let value = if value > 0 { Binary::On } else { Binary::Off };
                let value = Enumerated::Binary(value);
                Ok(SimpleApplicationDataValue::Enumerated(value))
            }

            x => Err(Error::Unimplemented(Unimplemented::ApplicationTagNumber(
                x.clone(),
            ))),
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        match self {
            Self::Boolean(x) => writer.push(*x as u8),
            Self::SignedInt(x) => writer.extend_from_slice(&x.to_be_bytes()),
            Self::UnsignedInt(x) => writer.extend_from_slice(&x.to_be_bytes()),
            Self::Real(x) => writer.extend_from_slice(&x.to_be_bytes()),
            Self::Double(x) => writer.extend_from_slice(&x.to_be_bytes()),
            Self::Enumerated(Enumerated::Binary(x)) => writer.push(x.clone() as u32 as u8),
            x => unimplemented!("{:?}", x),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeValue {
    pub time: Time,
    pub value: SimpleApplicationDataValue,
}

impl TimeValue {
    pub const LEN: u32 = 4;

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<TimeValue, Error> {
        let tag = Tag::decode_expected(
            reader,
            buf,
            TagNumber::Application(ApplicationTagNumber::Time),
            "Decode weekly schedule TimeValue",
        )?;

        // 4 bytes
        if tag.value != Self::LEN {
            return Err(Error::Length((
                "time tag should have length value 4",
                tag.value,
            )));
        }
        let time = match &tag.number {
            TagNumber::Application(ApplicationTagNumber::Time) => Time::decode(reader, buf)?,
            number => {
                return Err(Error::TagNotSupported((
                    "TimeValue decode time application tag expected",
                    number.clone(),
                )))
            }
        };
        let tag = Tag::decode(reader, buf)?;
        let value = SimpleApplicationDataValue::decode(&tag, reader, buf)?;
        Ok(TimeValue { time, value })
    }

    pub fn encode(&self, writer: &mut Writer) {
        let tag = Tag::new(
            TagNumber::Application(ApplicationTagNumber::Time),
            Self::LEN,
        );
        tag.encode(writer);
        self.time.encode(writer);
        let tag = self.value.tag();
        tag.encode(writer);
        self.value.encode(writer);
    }
}
