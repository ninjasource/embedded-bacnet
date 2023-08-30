use arrayref::array_ref;

use super::{
    error::Error,
    io::{Reader, Writer},
    object_id::ObjectId,
    property_id::PropertyId,
    tag::{ApplicationTagNumber, Tag, TagNumber},
};

// This gives you the bytes that begin after the opening tag and end before the closing tag
pub fn get_tagged_body<'a>(reader: &mut Reader, buf: &'a [u8]) -> (&'a [u8], u8) {
    let tag = Tag::decode(reader, buf);
    let tag_number = match &tag.number {
        TagNumber::ContextSpecificOpening(x) => *x,
        _ => panic!("Expected opening tag but got: {:?}", tag),
    };

    let index = reader.index;
    let mut counter = 0;
    loop {
        let tag = Tag::decode(reader, buf);

        // keep track of nested tags and when we reach our last closing tag then we are done
        match tag.number {
            TagNumber::ContextSpecificOpening(x) if x == tag_number => counter += 1,
            TagNumber::ContextSpecificClosing(x) if x == tag_number => {
                if counter == 0 {
                    //  let len = reader.index - index - 1;
                    let end = reader.index - 1; // -1 to ignore the last closing tag
                    return (&buf[index..end], tag_number);
                } else {
                    counter -= 1;
                }
            }
            TagNumber::Application(ApplicationTagNumber::Boolean) => {
                // tag value is not a length for bool
            }
            _ => {
                // skip past value and read next tag
                reader.index += tag.value as usize;
            }
        }
    }
}

pub fn encode_u16(writer: &mut Writer, value: u16) {
    writer.extend_from_slice(&value.to_be_bytes());
}

pub fn encode_u24(writer: &mut Writer, value: u32) {
    let slice = &value.to_be_bytes();
    writer.extend_from_slice(&slice[..3]);
}

pub fn encode_u32(writer: &mut Writer, value: u32) {
    writer.extend_from_slice(&value.to_be_bytes());
}

pub fn encode_u64(writer: &mut Writer, value: u64) {
    writer.extend_from_slice(&value.to_be_bytes());
}

pub fn _parse_unsigned(bytes: &[u8], len: u32) -> Result<(&[u8], u32), Error> {
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

pub fn encode_context_object_id(writer: &mut Writer, tag_number: u8, object_id: &ObjectId) {
    let tag = Tag::new(TagNumber::ContextSpecific(tag_number), ObjectId::LEN);
    tag.encode(writer);
    object_id.encode(writer);
}

pub fn encode_context_bool(writer: &mut Writer, tag_number: u8, value: bool) {
    const LEN: u32 = 1; // 1 byte
    let tag = Tag::new(TagNumber::ContextSpecific(tag_number), LEN);
    tag.encode(writer);
    let item = if value { 1 } else { 0 };
    writer.push(item);
}

pub fn encode_opening_tag(writer: &mut Writer, tag_number: u8) {
    if tag_number <= 14 {
        let byte = 0b0001000 | (tag_number << 4) | 6;
        writer.push(byte)
    } else {
        let byte = 0b0001000 | 0xF0 | 6;
        writer.push(byte);
        writer.push(tag_number)
    }
}

pub fn encode_closing_tag(writer: &mut Writer, tag_number: u8) {
    if tag_number <= 14 {
        let byte = 0b0001000 | (tag_number << 4) | 7;
        writer.push(byte)
    } else {
        let byte = 0b0001000 | 0xF0 | 7;
        writer.push(byte);
        writer.push(tag_number)
    }
}

pub fn encode_context_unsigned(writer: &mut Writer, tag_number: u8, value: u32) {
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
    tag.encode(writer);
    encode_unsigned(writer, value as u64);
}

pub fn decode_context_enumerated(reader: &mut Reader, buf: &[u8]) -> (Tag, PropertyId) {
    let tag = Tag::decode(reader, buf);
    let property_id: PropertyId = (decode_unsigned(tag.value, reader, buf) as u32).into();

    (tag, property_id)
}

pub fn encode_context_enumerated(writer: &mut Writer, tag_number: u8, property_id: PropertyId) {
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
    tag.encode(writer);
    encode_unsigned(writer, value as u64);
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

pub fn encode_unsigned(writer: &mut Writer, value: u64) {
    if value < 0x100 {
        writer.push(value as u8);
    } else if value < 0x10000 {
        encode_u16(writer, value as u16);
    } else if value < 0x1000000 {
        encode_u24(writer, value as u32);
    } else if value < 0x100000000 {
        encode_u32(writer, value as u32);
    } else {
        encode_u64(writer, value)
    }
}
