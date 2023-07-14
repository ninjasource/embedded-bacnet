use arrayref::array_ref;

use super::{error::Error, object_id::ObjectId, property_id::PropertyId};
use crate::common::tag::{Tag, TagNumber};

pub struct Buffer<'a> {
    pub buf: &'a mut [u8],
    pub index: usize,
}

impl<'a> Buffer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, index: 0 }
    }

    pub fn push(&mut self, item: u8) {
        self.buf[self.index] = item;
        self.index += 1;
    }

    pub fn extend_from_slice(&mut self, src: &[u8]) {
        assert!(src.len() <= self.buf.len() - self.index);
        self.buf[self.index..self.index + src.len()].copy_from_slice(src);
        self.index += src.len();
    }

    pub fn to_bytes(&self) -> &[u8] {
        &self.buf[..self.index]
    }
}

pub struct Reader {
    index: usize,
    payload_len: usize,
    len: usize,
}

impl Reader {
    pub fn eof(&self) -> bool {
        self.index == self.len
    }

    pub fn new(payload_len: usize) -> Self {
        Self {
            index: 0,
            payload_len,
            len: usize::MAX - 1000,
        }
    }

    pub fn set_len(&mut self, len: usize) -> Result<(), Error> {
        if len > self.payload_len {
            Err(Error::Length(
                "read buffer too small to fit entire bacnet payload",
            ))
        } else {
            self.len = len;
            Ok(())
        }
    }

    pub fn read_byte(&mut self, buf: &[u8]) -> u8 {
        if self.eof() {
            panic!("read_byte attempt to read past end of buffer");
        } else {
            let byte = buf[self.index];
            self.index += 1;
            byte
        }
    }

    pub fn read_bytes<const COUNT: usize>(&mut self, buf: &[u8]) -> [u8; COUNT] {
        if self.index + COUNT >= self.len {
            panic!("read_bytes attempt to read past end of buffer");
        } else {
            let mut tmp: [u8; COUNT] = [0; COUNT];
            tmp.copy_from_slice(&buf[self.index..self.index + COUNT]);
            self.index += COUNT;
            tmp
        }
    }

    pub fn read_slice<'a>(&mut self, len: usize, buf: &'a [u8]) -> &'a [u8] {
        if self.index + len >= self.len {
            panic!("read_slice attempt to read past end of buffer");
        } else {
            let slice = &buf[self.index..self.index + len];
            self.index += len;
            slice
        }
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

pub fn encode_context_object_id(buffer: &mut Buffer, tag_number: u8, object_id: &ObjectId) {
    let tag = Tag::new(TagNumber::ContextSpecific(tag_number), 4);
    tag.encode(buffer);
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

    let tag = Tag::new(TagNumber::ContextSpecific(tag_number), len);
    tag.encode(buffer);
    encode_unsigned(buffer, value as u64);
}

pub fn decode_context_enumerated(reader: &mut Reader, buf: &[u8]) -> (Tag, PropertyId) {
    let tag = Tag::decode(reader, buf);
    let property_id: PropertyId = (decode_unsigned(tag.value, reader, buf) as u32).into();

    (tag, property_id)
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

    let tag = Tag::new(TagNumber::ContextSpecific(tag_number), len);
    tag.encode(buffer);
    encode_unsigned(buffer, value as u64);
}

pub fn decode_unsigned(len: u32, reader: &mut Reader, buf: &[u8]) -> u64 {
    match len {
        1 => reader.read_byte(buf) as u64,
        2 => u16::from_be_bytes(reader.read_bytes(buf)) as u64,
        3 => {
            let bytes: [u8; 3] = reader.read_bytes(buf);
            let mut tmp: [u8; 4] = [0; 4];
            tmp[1..].copy_from_slice(&bytes);
            u32::from_be_bytes(tmp) as u64
        }

        4 => u32::from_be_bytes(reader.read_bytes(buf)) as u64,
        8 => u64::from_be_bytes(reader.read_bytes(buf)) as u64,
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
