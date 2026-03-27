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

fn encode_unsigned_impl(writer: &mut Writer, tag_number: TagNumber, value: impl Into<u64>) {
    let data = value.into().to_be_bytes();
    let skip = data.into_iter().take_while(|&x| x == 0).count();
    let data = &data[skip..];
    Tag::new(tag_number, data.len() as u32)
        .encode(writer);
    writer.extend_from_slice(data);
}

pub fn encode_unsigned(writer: &mut Writer, context_tag: Option<u8>, value: impl Into<u64>) {
    let tag_number = context_tag.map(TagNumber::ContextSpecific)
        .unwrap_or(TagNumber::Application(ApplicationTagNumber::UnsignedInt));
    encode_unsigned_impl(writer, tag_number, value);
}

pub fn encode_signed(writer: &mut Writer, context_tag: Option<u8>, value: impl Into<i64>) {
    let data = value.into().to_be_bytes();
    let leading_zeros = data.into_iter().take_while(|&x| x == 0).count();
    let leading_signs = data.into_iter().take_while(|&x| x == 0xFF).count();
    let skip = usize::max(leading_zeros, leading_signs);
    let data = &data[skip..];

    let tag_number = context_tag.map(TagNumber::ContextSpecific)
        .unwrap_or(TagNumber::Application(ApplicationTagNumber::SignedInt));
    Tag::new(tag_number, data.len() as u32)
        .encode(writer);
    writer.extend_from_slice(data);
}

pub fn encode_enumerated(writer: &mut Writer, value: impl Into<u64>, context_tag: Option<u8>) {
    let tag_number = context_tag.map(TagNumber::ContextSpecific)
        .unwrap_or(TagNumber::Application(ApplicationTagNumber::Enumerated));
    encode_unsigned_impl(writer, tag_number, value);
}

pub fn encode_application_unsigned(writer: &mut Writer, value: impl Into<u64>) {
    encode_unsigned(writer, None, value);
}

pub fn encode_context_unsigned(writer: &mut Writer, tag_number: u8, value: impl Into<u64>) {
    encode_unsigned(writer, Some(tag_number), value);
}

pub fn encode_application_signed(writer: &mut Writer, value: impl Into<i64>) {
    encode_signed(writer, None, value);
}

pub fn encode_context_signed(writer: &mut Writer, tag_number: u8, value: impl Into<i64>) {
    encode_signed(writer, Some(tag_number), value);
}

pub fn encode_application_enumerated(writer: &mut Writer, value: u32) {
    encode_enumerated(writer, value, None);
}

pub fn encode_context_enumerated(writer: &mut Writer, tag_number: u8, property_id: &PropertyId) {
    encode_enumerated(writer, *property_id as u32, Some(tag_number))
}

pub fn encode_application_object_id(writer: &mut Writer, object_id: &ObjectId) {
    Tag::new(
        TagNumber::Application(ApplicationTagNumber::ObjectId),
        ObjectId::LEN,
    )
    .encode(writer);
    object_id.encode(writer);
}

pub fn decode_unsigned(len: u32, reader: &mut Reader, buf: &[u8]) -> Result<u64, Error> {
    if len > 8 {
        return Err(Error::Length(("integers bigger than 64 bits bytes are not supported", len)));
    }
    let len = len as usize;
    let mut bytes = [0; 8];
    bytes[..len].copy_from_slice(reader.read_slice(len, buf)?);
    let value = u64::from_be_bytes(bytes) >> (8 * (8 - len));
    Ok(value)
}

pub fn decode_signed(len: u32, reader: &mut Reader, buf: &[u8]) -> Result<i64, Error> {
    if len > 8 {
        return Err(Error::Length(("integers bigger than 64 bits are not supported", len)));
    }
    // Read into the most significant bits, then do a right shift to get sign extension for negative integers.
    let len = len as usize;
    let mut bytes = [0; 8];
    bytes[..len].copy_from_slice(reader.read_slice(len, buf)?);
    let value = i64::from_be_bytes(bytes) >> (8 * (8 - len));
    Ok(value)
}
