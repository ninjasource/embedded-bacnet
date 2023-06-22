use heapless::Vec;

use crate::common::{
    helper::{encode_u16, encode_u24, encode_u32, encode_u64, Buffer, Reader},
    object_id::ObjectId,
    property_id::PropertyId,
    spec::BACNET_ARRAY_ALL,
    tag::Tag,
};

#[derive(Debug)]
pub struct ReadProperty {
    pub object_id: ObjectId,     // e.g ObjectDevice:20088
    pub property_id: PropertyId, // e.g. PropObjectList
    pub array_index: u32,        // use BACNET_ARRAY_ALL for all
    pub properties: Option<Vec<ObjectId, 512>>,
}

impl ReadProperty {
    pub fn new(object_id: ObjectId, property_id: PropertyId) -> Self {
        Self {
            object_id,
            property_id,
            array_index: BACNET_ARRAY_ALL,
            properties: None,
        }
    }

    pub fn encode(&self, buffer: &mut Buffer) {
        // object_id
        encode_context_object_id(buffer, 0, &self.object_id);

        // property_id
        encode_context_enumerated(buffer, 1, self.property_id);

        // array_index
        if self.array_index != BACNET_ARRAY_ALL {
            encode_context_unsigned(buffer, 2, self.array_index);
        }
    }

    pub fn decode(reader: &mut Reader) -> Self {
        let object_id = decode_context_object_id(reader);
        let (tag_num, property_id) = decode_context_enumerated(reader);
        assert_eq!(tag_num, 1, "invalid property id tag");

        match property_id {
            PropertyId::PropObjectList => {
                let tag = Tag::decode(reader);
                assert_eq!(tag.number, 3, "expected opening tag");

                let mut properties = Vec::new();

                loop {
                    let tag = Tag::decode(reader);
                    if tag.number == 3 {
                        // closing tag
                        break;
                    }

                    let object_id = ObjectId::decode(reader, tag.value).unwrap();
                    properties.push(object_id).unwrap();
                }

                return Self {
                    object_id,
                    property_id,
                    array_index: BACNET_ARRAY_ALL,
                    properties: Some(properties),
                };
            }
            _ => unimplemented!(),
        }
    }
}

fn decode_context_object_id(reader: &mut Reader) -> ObjectId {
    let tag = Tag::decode(reader);
    assert_eq!(tag.number, 0, "unexpected object_id tag number");

    ObjectId::decode(reader, tag.value).unwrap()
}

fn encode_context_object_id(buffer: &mut Buffer, tag_number: u8, object_id: &ObjectId) {
    let tag = Tag::new(tag_number, 4);
    tag.encode(true, buffer);
    object_id.encode(buffer);
}

fn encode_context_unsigned(buffer: &mut Buffer, tag_number: u8, value: u32) {
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

fn decode_context_enumerated(reader: &mut Reader) -> (u8, PropertyId) {
    let tag = Tag::decode(reader);
    let property_id: PropertyId = (decode_unsigned(reader, tag.value) as u32).into();

    (tag.number, property_id)
}

fn encode_context_enumerated(buffer: &mut Buffer, tag_number: u8, property_id: PropertyId) {
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

fn decode_unsigned(reader: &mut Reader, len: u32) -> u64 {
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

fn encode_unsigned(buffer: &mut Buffer, value: u64) {
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
