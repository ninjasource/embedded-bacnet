use crate::common::{
    error::Error,
    io::{Reader, Writer},
    object_id::ObjectId,
    property_id::PropertyId,
    tag::{ApplicationTagNumber, Tag, TagNumber},
};

// reads and checks the opening tag number passed in
pub fn get_tagged_body_for_tag<'a>(
    reader: &mut Reader,
    buf: &'a [u8],
    expected_tag_number: u8,
    context: &'static str,
) -> Result<&'a [u8], Error> {
    Tag::decode_expected(
        reader,
        buf,
        TagNumber::ContextSpecificOpening(expected_tag_number),
        context,
    )?;

    get_tagged_body_internal(reader, buf, expected_tag_number)
}

// This gives you the bytes that begin after the opening tag and end before the closing tag
pub fn get_tagged_body<'a>(reader: &mut Reader, buf: &'a [u8]) -> Result<(&'a [u8], u8), Error> {
    let tag = Tag::decode(reader, buf)?;
    let tag_number = match tag.number {
        TagNumber::ContextSpecificOpening(x) => x,
        x => return Err(Error::ExpectedOpeningTag(x)),
    };

    let buf = get_tagged_body_internal(reader, buf, tag_number)?;
    Ok((buf, tag_number))
}

fn get_tagged_body_internal<'a>(
    reader: &mut Reader,
    buf: &'a [u8],
    opening_tag_number: u8,
) -> Result<&'a [u8], Error> {
    let index = reader.index;
    let mut counter = 0;
    loop {
        let tag = Tag::decode(reader, buf)?;

        // keep track of nested tags and when we reach our last closing tag then we are done
        match tag.number {
            TagNumber::ContextSpecificOpening(x) if x == opening_tag_number => counter += 1,
            TagNumber::ContextSpecificClosing(x) if x == opening_tag_number => {
                if counter == 0 {
                    //  let len = reader.index - index - 1;
                    let end = reader.index - 1; // -1 to ignore the last closing tag
                    return Ok(&buf[index..end]);
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

pub fn encode_i16(writer: &mut Writer, value: i16) {
    writer.extend_from_slice(&value.to_be_bytes());
}

pub fn encode_i32(writer: &mut Writer, value: i32) {
    writer.extend_from_slice(&value.to_be_bytes());
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

pub fn encode_context_object_id(writer: &mut Writer, tag_number: u8, object_id: &ObjectId) {
    let tag = Tag::new(TagNumber::ContextSpecific(tag_number), ObjectId::LEN);
    tag.encode(writer);
    object_id.encode(writer);
}

pub fn decode_context_object_id(
    reader: &mut Reader,
    buf: &[u8],
    expected_tag_num: u8,
    context: &'static str,
) -> Result<ObjectId, Error> {
    let tag = Tag::decode_expected(
        reader,
        buf,
        TagNumber::ContextSpecific(expected_tag_num),
        context,
    )?;
    let object_id = ObjectId::decode(tag.value, reader, buf)?;
    Ok(object_id)
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
    let len = get_len_u64(value as u64);

    let tag = Tag::new(TagNumber::ContextSpecific(tag_number), len);
    tag.encode(writer);
    encode_unsigned(writer, len, value as u64);
}

pub fn decode_context_property_id(
    reader: &mut Reader,
    buf: &[u8],
    expected_tag_number: u8,
    context: &'static str,
) -> Result<PropertyId, Error> {
    let tag = Tag::decode_expected(
        reader,
        buf,
        TagNumber::ContextSpecific(expected_tag_number),
        context,
    )?;
    let property_id: PropertyId = (decode_unsigned(tag.value, reader, buf)? as u32).into();

    Ok(property_id)
}

pub fn encode_context_enumerated(writer: &mut Writer, tag_number: u8, property_id: &PropertyId) {
    let value = *property_id as u32;
    let len = get_len_u64(value as u64);

    let tag = Tag::new(TagNumber::ContextSpecific(tag_number), len);
    tag.encode(writer);
    encode_unsigned(writer, len, value as u64);
}

pub fn encode_application_unsigned(writer: &mut Writer, value: u64) {
    let len = get_len_u64(value);
    Tag::new(
        TagNumber::Application(ApplicationTagNumber::UnsignedInt),
        len,
    )
    .encode(writer);
    encode_unsigned(writer, len, value);
}

pub fn encode_application_enumerated(writer: &mut Writer, value: u32) {
    let len = get_len_u32(value);
    let tag = Tag::new(
        TagNumber::Application(ApplicationTagNumber::Enumerated),
        len,
    );
    tag.encode(writer);
    encode_unsigned(writer, len, value as u64);
}

pub fn encode_application_object_id(writer: &mut Writer, object_id: &ObjectId) {
    Tag::new(
        TagNumber::Application(ApplicationTagNumber::ObjectId),
        ObjectId::LEN,
    )
    .encode(writer);
    object_id.encode(writer);
}

pub fn encode_application_signed(writer: &mut Writer, value: i32) {
    let mut len = get_len_i32(value);
    len = if len == 3 { 4 } else { len }; // we don't bother with 3 byte integers (just save it as a 4 byte integer)
    Tag::new(TagNumber::Application(ApplicationTagNumber::SignedInt), len).encode(writer);
    encode_signed(writer, len, value);
}

pub fn get_len_u32(value: u32) -> u32 {
    if value < 0x100 {
        1
    } else if value < 0x10000 {
        2
    } else if value < 0x1000000 {
        3
    } else {
        4
    }
}

fn get_len_u64(value: u64) -> u32 {
    if value < 0x100 {
        1
    } else if value < 0x10000 {
        2
    } else if value < 0x1000000 {
        3
    } else if value < 0x100000000 {
        4
    } else {
        8
    }
}

fn get_len_i32(value: i32) -> u32 {
    if (-128..128).contains(&value) {
        1
    } else if (-32768..32768).contains(&value) {
        2
    } else if (-8388608..8388608).contains(&value) {
        3
    } else {
        4
    }
}

pub fn decode_unsigned(len: u32, reader: &mut Reader, buf: &[u8]) -> Result<u64, Error> {
    let value = match len {
        1 => reader.read_byte(buf)? as u64,
        2 => u16::from_be_bytes(reader.read_bytes(buf)?) as u64,
        3 => {
            let bytes: [u8; 3] = reader.read_bytes(buf)?;
            let mut tmp: [u8; 4] = [0; 4];
            tmp[1..].copy_from_slice(&bytes);
            u32::from_be_bytes(tmp) as u64
        }
        4 => u32::from_be_bytes(reader.read_bytes(buf)?) as u64,
        8 => u64::from_be_bytes(reader.read_bytes(buf)?),
        x => return Err(Error::Length(("unsigned len must be between 1 and 8", x))),
    };

    Ok(value)
}

pub fn _decode_u32(len: u32, reader: &mut Reader, buf: &[u8]) -> Result<u32, Error> {
    let value = match len {
        1 => reader.read_byte(buf)? as u32,
        2 => u16::from_be_bytes(reader.read_bytes(buf)?) as u32,
        3 => {
            let bytes: [u8; 3] = reader.read_bytes(buf)?;
            let mut tmp: [u8; 4] = [0; 4];
            tmp[1..].copy_from_slice(&bytes);
            u32::from_be_bytes(tmp)
        }
        4 => u32::from_be_bytes(reader.read_bytes(buf)?),
        x => return Err(Error::Length(("u32 len must be between 1 and 4", x))),
    };

    Ok(value)
}

pub fn decode_signed(len: u32, reader: &mut Reader, buf: &[u8]) -> Result<i32, Error> {
    let value = match len {
        1 => reader.read_byte(buf)? as i32,
        2 => u16::from_be_bytes(reader.read_bytes(buf)?) as i32,
        3 => {
            let bytes: [u8; 3] = reader.read_bytes(buf)?;
            let mut tmp: [u8; 4] = [0; 4];
            tmp[1..].copy_from_slice(&bytes);
            i32::from_be_bytes(tmp)
        }
        4 => i32::from_be_bytes(reader.read_bytes(buf)?),
        x => return Err(Error::Length(("signed len must be between 1 and 4", x))),
    };

    Ok(value)
}

pub fn encode_unsigned(writer: &mut Writer, len: u32, value: u64) {
    match len {
        1 => writer.push(value as u8),
        2 => encode_u16(writer, value as u16),
        3 => encode_u24(writer, value as u32),
        4 => encode_u32(writer, value as u32),
        8 => encode_u64(writer, value),
        _ => unreachable!(),
    }
}

pub fn encode_signed(writer: &mut Writer, len: u32, value: i32) {
    match len {
        1 => writer.push(value as u8),
        2 => encode_i16(writer, value as i16),
        4 => encode_i32(writer, value),
        _ => unreachable!(),
    }
}
