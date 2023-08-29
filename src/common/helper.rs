use arrayref::array_ref;
use log::info;

use super::{
    error::Error,
    object_id::{ObjectId, ObjectType},
    property_id::PropertyId,
    tag::ApplicationTagNumber,
};
use crate::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestSerivice},
        primitives::data_value::ApplicationDataValue,
        services::read_property::{ReadProperty, ReadPropertyValue},
    },
    common::tag::{Tag, TagNumber},
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
    },
};

pub struct Writer<'a> {
    pub buf: &'a mut [u8],
    pub index: usize,
}

impl<'a> Writer<'a> {
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

#[derive(Debug)]
pub struct Reader {
    pub index: usize,
    pub end: usize,
}

impl Reader {
    pub fn eof(&self) -> bool {
        self.index >= self.end
    }

    pub fn new() -> Self {
        Self {
            index: 0,
            end: usize::MAX - 1000,
        }
    }

    pub fn set_len(&mut self, len: usize) {
        self.end = len;
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
        if self.index + COUNT > self.end {
            panic!("read_bytes attempt to read past end of buffer");
        } else {
            let mut tmp: [u8; COUNT] = [0; COUNT];
            tmp.copy_from_slice(&buf[self.index..self.index + COUNT]);
            self.index += COUNT;
            tmp
        }
    }

    pub fn read_slice<'a>(&mut self, len: usize, buf: &'a [u8]) -> &'a [u8] {
        if self.index + len > self.end {
            panic!("read_slice attempt to read past end of buffer");
        } else {
            let slice = &buf[self.index..self.index + len];
            self.index += len;
            slice
        }
    }
}

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

/*
// This gives you a reader that begins after the opening tag and ends before the closing tag
pub fn get_tagged_body(expected_tag_number: u8, reader: &mut Reader, buf: &[u8]) -> Reader {
    let tag = Tag::decode(reader, buf);
    let tag_number = match &tag.number {
        TagNumber::ContextSpecificOpening(expected_tag_number) => *expected_tag_number,
        _ => panic!(
            "Expected opening tag {} but got: {:?}",
            expected_tag_number, tag
        ),
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
                    return Reader { index, end };
                } else {
                    counter -= 1;
                }
            }
            _ => {
                // ignore all other tags
            }
        }

        // skip past value and read next tag
        reader.index += tag.value as usize;
    }
}*/

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

pub fn read_property_req_to_backnet(
    invoke_id: u8,
    object_id: ObjectId,
    property_id: PropertyId,
    buf: &mut [u8],
) -> usize {
    let read_property = ReadProperty::new(object_id, property_id);
    let req = ConfirmedRequest::new(
        invoke_id,
        ConfirmedRequestSerivice::ReadProperty(read_property),
    );

    req_to_bacnet(req, buf)
}

pub fn req_to_bacnet(req: ConfirmedRequest<'_>, buf: &mut [u8]) -> usize {
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));
    let mut writer = Writer::new(buf);
    data_link.encode(&mut writer);
    writer.index
}

// a helper shortcut function to get a read property string result from a backnet-ip packet
pub fn bacnet_to_string<'a>(buf: &'a [u8]) -> &'a str {
    let mut reader = Reader::new();
    let message: DataLink<'a> = DataLink::decode(&mut reader, buf).unwrap();

    if let Some(ack) = message.get_read_property_ack_into() {
        if let ReadPropertyValue::ApplicationDataValue(ApplicationDataValue::CharacterString(x)) =
            &ack.property_value
        {
            let s = x.inner;
            return s;
        }
    }

    return "";
}

pub trait ReadWrite {
    fn recv(&self, buf: &mut [u8]) -> Result<usize, Error>;
    fn send(&self, buf: &[u8]) -> Result<(), Error>;
}

pub struct BacnetService<T: ReadWrite> {
    object_id: u32,
    io: T,
    invoke_id: u8,
}

impl<T: ReadWrite> BacnetService<T> {
    pub fn new(io: T, object_id: u32) -> Self {
        let invoke_id = 0;
        Self {
            object_id,
            io,
            invoke_id,
        }
    }

    pub fn read_string<'a>(
        &mut self,
        property_id: PropertyId,
        buf: &'a mut [u8],
    ) -> Result<&'a str, Error> {
        let invoke_id = self.invoke_id;
        self.invoke_id = self.invoke_id.wrapping_add(1);

        // encode packet
        let object_id = ObjectId::new(ObjectType::ObjectDevice, self.object_id);
        let read_property = ReadProperty::new(object_id, property_id);
        let req = ConfirmedRequest::new(
            invoke_id,
            ConfirmedRequestSerivice::ReadProperty(read_property),
        );
        let apdu = ApplicationPdu::ConfirmedRequest(req);
        let src = None;
        let dst = None;
        let message = NetworkMessage::Apdu(apdu);
        let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
        let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));
        let mut buffer = Writer::new(buf);
        data_link.encode(&mut buffer);

        // send packet
        let send_buf = buffer.to_bytes();
        info!("Sending: {:?}", send_buf);
        self.io.send(send_buf)?;

        // receive reply
        let n = self.io.recv(buf)?;
        let mut reader = Reader::new();
        let message: DataLink<'a> = DataLink::decode(&mut reader, &buf[..n]).unwrap();

        // check that the request and response invoke ids match
        if let Some(ack) = message.get_ack() {
            if ack.invoke_id != invoke_id {
                panic!("Invalid invoke id")
            }
        }

        if let Some(ack) = message.get_read_property_ack_into() {
            if let ReadPropertyValue::ApplicationDataValue(ApplicationDataValue::CharacterString(
                x,
            )) = &ack.property_value
            {
                let s = x.inner;
                return Ok(s);
            }
        }

        Ok("")
    }
}
