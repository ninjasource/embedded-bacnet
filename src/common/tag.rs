use crate::common::{
    error::{Error, ExpectedTag},
    io::{Reader, Writer},
};

// byte0:
// bits 7-4 tag_num
// bit  3   class (0 = application tag_num, 1 = context specific tag_num)
// bits 2-0 length / value / type
//
// Can use additional bytes as specified in bits 2-0 above

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum ApplicationTagNumber {
    Null = 0,
    Boolean = 1,
    UnsignedInt = 2,
    SignedInt = 3,
    Real = 4,
    Double = 5,
    OctetString = 6,
    CharacterString = 7,
    BitString = 8,
    Enumerated = 9,
    Date = 10,
    Time = 11,
    ObjectId = 12,
    Reserve1 = 13,
    Reserve2 = 14,
    Reserve3 = 15,
}

impl From<u8> for ApplicationTagNumber {
    fn from(tag_number: u8) -> Self {
        match tag_number {
            0 => Self::Null,
            1 => Self::Boolean,
            2 => Self::UnsignedInt,
            3 => Self::SignedInt,
            4 => Self::Real,
            5 => Self::Double,
            6 => Self::OctetString,
            7 => Self::CharacterString,
            8 => Self::BitString,
            9 => Self::Enumerated,
            10 => Self::Date,
            11 => Self::Time,
            12 => Self::ObjectId,
            13 => Self::Reserve1,
            14 => Self::Reserve2,
            15 => Self::Reserve3,
            _ => unreachable!(), // tag_number is only 4 bits
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TagNumber {
    Application(ApplicationTagNumber),
    ContextSpecific(u8),
    ContextSpecificOpening(u8),
    ContextSpecificClosing(u8),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tag {
    pub number: TagNumber,
    pub value: u32,
}

impl Tag {
    pub fn new(number: TagNumber, value: u32) -> Self {
        Self { number, value }
    }

    pub fn encode(&self, writer: &mut Writer) {
        let mut buf: [u8; 10] = [0; 10];
        let mut len = 1;

        match &self.number {
            TagNumber::Application(num) => {
                buf[0] |= (num.clone() as u8) << 4;
            }
            TagNumber::ContextSpecificOpening(num) => {
                let num = *num;
                buf[0] |= 0b1000; // set class to context specific

                if num <= 14 {
                    buf[0] |= num << 4;
                } else {
                    buf[0] |= 0xF0;
                    buf[1] = num;
                    len += 1;
                }

                // set type field to opening tag
                buf[0] |= 6;
            }
            TagNumber::ContextSpecificClosing(num) => {
                let num = *num;
                buf[0] |= 0b1000; // set class to context specific

                if num <= 14 {
                    buf[0] |= num << 4;
                } else {
                    buf[0] |= 0xF0;
                    buf[1] = num;
                    len += 1;
                }

                // set type field to closing tag
                buf[0] |= 7;
            }
            TagNumber::ContextSpecific(num) => {
                let num = *num;
                buf[0] |= 0b1000; // set class to context specific

                if num <= 14 {
                    buf[0] |= num << 4;
                } else {
                    buf[0] |= 0xF0;
                    buf[1] = num;
                    len += 1;
                }
            }
        }

        if self.value <= 4 {
            buf[0] |= self.value as u8;
        } else {
            buf[0] |= 5;

            if self.value <= 253 {
                buf[len] = self.value as u8;
                len += 1;
            } else if self.value < u16::MAX as u32 {
                buf[len] = self.value as u8;
                len += 1;
                let tmp = u16::to_be_bytes(self.value as u16);
                buf[len..len + tmp.len()].copy_from_slice(&tmp);
                len += tmp.len();
            } else {
                buf[len] = self.value as u8;
                len += 1;
                let tmp = u32::to_be_bytes(self.value);
                buf[len..len + tmp.len()].copy_from_slice(&tmp);
                len += tmp.len();
            }
        }

        writer.extend_from_slice(&buf[..len]);
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let (number, byte0) = decode_tag_number(reader, buf)?;

        let value = if is_extended_value(byte0) {
            let byte = reader.read_byte(buf)?;
            match byte {
                // tagged as u32
                255 => {
                    let bytes = reader.read_bytes(buf)?;
                    let value = u32::from_be_bytes(bytes);
                    Self { number, value }
                }
                // tagged as u16
                254 => {
                    let bytes = reader.read_bytes(buf)?;
                    let value = u16::from_be_bytes(bytes) as u32;
                    Self { number, value }
                }
                // no tag
                _ => Self {
                    number,
                    value: byte.into(),
                },
            }
        } else if is_opening_tag(byte0) | is_closing_tag(byte0) {
            Self { number, value: 0 }
        } else {
            let value = (byte0 & 0x07).into();
            Self { number, value }
        };

        Ok(value)
    }

    pub fn decode_expected(
        reader: &mut Reader,
        buf: &[u8],
        expected: TagNumber,
        context: &'static str,
    ) -> Result<Self, Error> {
        let tag = Self::decode(reader, buf)?;
        if tag.number == expected {
            Ok(tag)
        } else {
            Err(Error::ExpectedTag(ExpectedTag {
                context,
                expected,
                actual: tag.number,
            }))
        }
    }

    pub fn expect_value(&self, context: &'static str, value: u32) -> Result<(), Error> {
        if self.value != value {
            Err(Error::TagValueInvalid((context, self.clone(), value)))
        } else {
            Ok(())
        }
    }

    pub fn expect_number(&self, context: &'static str, tag_number: TagNumber) -> Result<(), Error> {
        if self.number == tag_number {
            Ok(())
        } else {
            Err(Error::ExpectedTag(ExpectedTag {
                actual: self.number.clone(),
                expected: tag_number,
                context,
            }))
        }
    }
}

// returns tag_number and byte0 because we need to reuse byte0 elsewhere
fn decode_tag_number(reader: &mut Reader, buf: &[u8]) -> Result<(TagNumber, u8), Error> {
    let byte0 = reader.read_byte(buf)?;

    let value = if is_context_specific(byte0) {
        // context specific tag num
        if is_extended_tag_number(byte0) {
            let num = reader.read_byte(buf)?;
            (TagNumber::ContextSpecific(num), byte0)
        } else {
            let num = byte0 >> 4;
            if is_opening_tag(byte0) {
                (TagNumber::ContextSpecificOpening(num), 0)
            } else if is_closing_tag(byte0) {
                (TagNumber::ContextSpecificClosing(num), 0)
            } else {
                (TagNumber::ContextSpecific(num), byte0)
            }
        }
    } else {
        // application tag num
        let num = (byte0 >> 4).into();
        (TagNumber::Application(num), byte0)
    };

    Ok(value)
}

fn is_extended_tag_number(byte0: u8) -> bool {
    byte0 & 0xF0 == 0xF0
}

fn is_extended_value(byte0: u8) -> bool {
    byte0 & 0x07 == 0x05
}

fn is_context_specific(byte0: u8) -> bool {
    byte0 & 0x08 == 0x08
}

fn is_opening_tag(byte0: u8) -> bool {
    byte0 & 0x07 == 0x06
}

fn is_closing_tag(byte0: u8) -> bool {
    byte0 & 0x07 == 0x07
}
