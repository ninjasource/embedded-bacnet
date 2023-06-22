use super::helper::{Buffer, Reader};

#[derive(Debug, PartialEq, Eq)]
pub enum TagType {
    Null,
    Boolean,
    UnsignedInt,
    SignedInt,
    Real,
    Double,
    OctetString,
    CharacterString,
    BitString,
    Enumerated,
    Date,
    Time,
    ObjectId,
    Reserve1,
    Reserve2,
    Reserve3,
    Unknown,
}

impl From<u8> for TagType {
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
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Tag {
    pub number: u8,
    pub value: u32,
}

impl Tag {
    pub fn new(number: u8, value: u32) -> Self {
        Self { number, value }
    }

    pub fn encode(&self, context_specific: bool, buffer: &mut Buffer) {
        let mut buf: [u8; 10] = [0; 10];
        let mut len = 1;

        if context_specific {
            buf[0] |= 0b1000
        }

        if self.number <= 14 {
            buf[0] |= self.number << 4;
        } else {
            buf[0] |= 0xF0;
            buf[1] = self.number;
            len += 1;
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

        buffer.extend_from_slice(&buf[..len]);
    }

    pub fn decode(reader: &mut Reader) -> Self {
        let (number, byte0) = decode_tag_number(reader);

        if is_extended_value(byte0) {
            let byte = reader.read_byte();
            match byte {
                // tagged as u32
                255 => {
                    let bytes = reader.read_bytes();
                    let value = u32::from_be_bytes(bytes);
                    Self { number, value }
                }
                // tagged as u16
                254 => {
                    let bytes = reader.read_bytes();
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
        }
    }

    pub fn tag_type(&self) -> TagType {
        self.number.into()
    }
}

// returns tag_number and byte0 because we need to reuse byte0 elsewhere
fn decode_tag_number(reader: &mut Reader) -> (u8, u8) {
    let byte0 = reader.read_byte();

    if is_extended_tag_number(byte0) {
        let num = reader.read_byte();
        (num, byte0)
    } else {
        let num = byte0 >> 4;
        (num, byte0)
    }
}

fn is_extended_tag_number(tagnum: u8) -> bool {
    tagnum & 0xF0 == 0xF0
}

fn is_extended_value(tagnum: u8) -> bool {
    tagnum & 0x07 == 5
}

fn is_context_specific(tagnum: u8) -> bool {
    tagnum & 0x08 == 0x08
}

fn is_opening_tag(tagnum: u8) -> bool {
    tagnum & 0x07 == 6
}

fn is_closing_tag(tagnum: u8) -> bool {
    tagnum & 0x07 == 7
}
