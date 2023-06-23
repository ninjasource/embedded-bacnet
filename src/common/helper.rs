use arrayref::array_ref;
use heapless::Vec;

use crate::common::tag::Tag;

use super::{error::Error, object_id::ObjectId, property_id::PropertyId};

pub struct Buffer {
    pub buf: Vec<u8, 1024>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn push(&mut self, item: u8) {
        self.buf.push(item).unwrap()
    }

    pub fn extend_from_slice(&mut self, src: &[u8]) {
        self.buf.extend_from_slice(src).unwrap()
    }

    pub fn to_bytes<'a>(&'a self) -> &'a [u8] {
        &self.buf
    }
}

pub struct Reader {
    buf: Vec<u8, 1024>,
    index: usize,
}

impl Reader {
    pub fn eof(&self) -> bool {
        self.index == self.buf.len()
    }

    pub fn new(payload: &[u8]) -> Self {
        let mut buf: Vec<u8, 1024> = Vec::new();
        buf.extend_from_slice(payload).unwrap();
        Self { buf, index: 0 }
    }

    pub fn read_byte(&mut self) -> u8 {
        let byte = self.buf[self.index];
        self.index += 1;
        byte
    }

    pub fn read_bytes<const COUNT: usize>(&mut self) -> [u8; COUNT] {
        let mut tmp: [u8; COUNT] = [0; COUNT];
        tmp.copy_from_slice(&self.buf[self.index..self.index + COUNT]);
        self.index += COUNT;
        tmp
    }

    pub fn read_slice<'a>(&'a mut self, len: usize) -> &'a [u8] {
        let slice = &self.buf[self.index..self.index + len];
        self.index += len;
        slice
    }
}

pub fn encode_u16(buffer: &mut Buffer, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn encode_u24(buffer: &mut Buffer, value: u32) {
    let slice = &value.to_be_bytes();
    buffer.extend_from_slice(&slice[..3]);
}

pub fn encode_u32(buffer: &mut Buffer, value: u32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn encode_u64(buffer: &mut Buffer, value: u64) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn parse_enumerated<T, E>(bytes: &[u8], len: u32) -> Result<(&[u8], T), T::Error>
where
    T: TryFrom<u32>,
{
    let (bytes, value) = parse_unsigned(bytes, len).unwrap();
    let value = T::try_from(value)?;
    Ok((bytes, value))
}

pub fn parse_unsigned(bytes: &[u8], len: u32) -> Result<(&[u8], u32), Error> {
    let len = len as usize;
    if len > 4 || len == 0 {
        return Err(Error::InvalidValue(
            "unsigned len value is 0 or greater than 4",
        ));
    }
    if bytes.len() < len {
        return Err(Error::Length(
            "unsigned len value greater than remaining bytes",
        ));
    }
    let val = match len {
        1 => bytes[0] as u32,
        2 => u16::from_be_bytes(*array_ref!(bytes, 0, 2)) as u32,
        3 => (bytes[0] as u32) << 16 | (bytes[1] as u32) << 8 | bytes[2] as u32,
        4 => u32::from_be_bytes(*array_ref!(bytes, 0, 4)),
        _ => panic!("invalid unsigned len"),
    };
    Ok((&bytes[len..], val))
}

pub fn decode_context_object_id(reader: &mut Reader) -> ObjectId {
    let tag = Tag::decode(reader);
    assert_eq!(tag.number, 0, "unexpected object_id tag number");

    ObjectId::decode(reader, tag.value).unwrap()
}

pub fn encode_context_object_id(buffer: &mut Buffer, tag_number: u8, object_id: &ObjectId) {
    let tag = Tag::new(tag_number, 4);
    tag.encode(true, buffer);
    object_id.encode(buffer);
}

pub fn encode_opening_tag(buffer: &mut Buffer, tag_number: u8) {
    if tag_number <= 14 {
        let byte = 0b0001000 | (tag_number << 4) | 6;
        buffer.push(byte)
    } else {
        let byte = 0b0001000 | 0xF0 | 6;
        buffer.push(byte);
        buffer.push(tag_number)
    }
}

pub fn encode_closing_tag(buffer: &mut Buffer, tag_number: u8) {
    if tag_number <= 14 {
        let byte = 0b0001000 | (tag_number << 4) | 7;
        buffer.push(byte)
    } else {
        let byte = 0b0001000 | 0xF0 | 7;
        buffer.push(byte);
        buffer.push(tag_number)
    }
}

pub fn encode_context_unsigned(buffer: &mut Buffer, tag_number: u8, value: u32) {
    let len = if value < 0x100 {
        1
    } else if value < 0x10000 {
        2
    } else if value < 0x1000000 {
        3
    } else {
        4
    };

    let tag = Tag::new(tag_number, len);
    tag.encode(true, buffer);
    encode_unsigned(buffer, value as u64);
}

pub fn decode_context_enumerated(reader: &mut Reader) -> (u8, PropertyId) {
    let tag = Tag::decode(reader);
    let property_id: PropertyId = (decode_unsigned(reader, tag.value) as u32).into();

    (tag.number, property_id)
}

pub fn encode_context_enumerated(buffer: &mut Buffer, tag_number: u8, property_id: PropertyId) {
    let value = property_id as u32;
    let len = if value < 0x100 {
        1
    } else if value < 0x10000 {
        2
    } else if value < 0x1000000 {
        3
    } else {
        4
    };

    let tag = Tag::new(tag_number, len);
    tag.encode(true, buffer);
    encode_unsigned(buffer, value as u64);
}

pub fn decode_unsigned(reader: &mut Reader, len: u32) -> u64 {
    match len {
        1 => reader.read_byte() as u64,
        2 => u16::from_be_bytes(reader.read_bytes()) as u64,
        3 => {
            let bytes: [u8; 3] = reader.read_bytes();
            let mut tmp: [u8; 4] = [0; 4];
            tmp[1..].copy_from_slice(&bytes);
            u32::from_be_bytes(tmp) as u64
        }

        4 => u32::from_be_bytes(reader.read_bytes()) as u64,
        8 => u64::from_be_bytes(reader.read_bytes()) as u64,
        _ => panic!("invalid unsigned len"),
    }
}

pub fn encode_unsigned(buffer: &mut Buffer, value: u64) {
    if value < 0x100 {
        buffer.push(value as u8);
    } else if value < 0x10000 {
        encode_u16(buffer, value as u16);
    } else if value < 0x1000000 {
        encode_u24(buffer, value as u32);
    } else if value < 0x100000000 {
        encode_u32(buffer, value as u32);
    } else {
        encode_u64(buffer, value)
    }
}
