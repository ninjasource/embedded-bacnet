use core::fmt::Display;

use alloc::{string::String, vec::Vec};

use crate::{
    application_protocol::primitives::data_value::ApplicationDataValue,
    common::{
        helper::{
            decode_unsigned, encode_closing_tag, encode_context_enumerated,
            encode_context_object_id, encode_context_unsigned, encode_opening_tag, Buffer, Reader,
        },
        object_id::ObjectId,
        property_id::PropertyId,
        spec::BACNET_ARRAY_ALL,
        tag::{Tag, TagNumber},
    },
};

#[derive(Debug)]
pub struct ReadPropertyMultipleAck {
    // pub objects: Vec<ObjectWithResults>,
}

#[derive(Debug)]
pub struct ObjectWithResults {
    pub object_id: ObjectId,
    // pub results: Vec<PropertyResult>,
}

impl ObjectWithResults {
    pub fn decode_next<'a>(
        &self,
        reader: &mut Reader,
        buf: &'a [u8],
    ) -> Option<PropertyResult<'a>> {
        let tag = Tag::decode(reader, buf);
        if tag.number == TagNumber::ContextSpecific(1) {
            // closing tag
            return None;
        }

        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(2),
            "expected property identifier tag"
        );
        let property_id: PropertyId = (decode_unsigned(tag.value, reader, buf) as u32).into();

        let tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(4),
            "expected opening tag"
        );

        let property_value = if property_id == PropertyId::PropEventTimeStamps {
            // hack to read past complicated timestamps
            loop {
                let byte = reader.read_byte(buf);
                // read until we get to the closing tag
                if byte == 0x4f {
                    break PropertyValue::PropValue(ApplicationDataValue::Boolean(false));
                }
            }
        } else {
            let tag = Tag::decode(reader, buf);
            let value =
                ApplicationDataValue::decode(&tag, &self.object_id, &property_id, reader, buf);
            let property_value = PropertyValue::PropValue(value);

            let tag = Tag::decode(reader, buf);
            assert_eq!(
                tag.number,
                TagNumber::ContextSpecific(4),
                "expected closing tag"
            );

            property_value
        };

        let property_result = PropertyResult {
            id: property_id,
            value: property_value,
        };

        Some(property_result)
    }
}

#[derive(Debug)]
pub struct PropertyResult<'a> {
    pub id: PropertyId,
    pub value: PropertyValue<'a>,
}

#[derive(Debug)]
pub enum PropertyValue<'a> {
    PropValue(ApplicationDataValue<'a>),
    PropDescription(&'a str),
    PropObjectName(String),
}

impl<'a> Display for PropertyValue<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self {
            Self::PropValue(x) => write!(f, "{}", x),
            _ => write!(f, "property value unprintable",),
        }
    }
}

impl ReadPropertyMultipleAck {
    pub fn decode_next(&self, reader: &mut Reader, buf: &[u8]) -> Option<ObjectWithResults> {
        if reader.eof() {
            return None;
        }

        let tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(0),
            "expected object_id tag"
        );
        let object_id = ObjectId::decode(tag.value, reader, buf).unwrap();

        //let object_id = decode_context_object_id(reader);
        let tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(1),
            "expected list of results opening tag"
        );

        let object_with_results = ObjectWithResults { object_id };
        Some(object_with_results)
    }
}

#[derive(Debug)]
pub struct ReadPropertyMultiple {
    pub objects: Vec<ReadPropertyMultipleObject>,
    pub array_index: u32, // use BACNET_ARRAY_ALL for all
}

#[derive(Debug)]
pub struct ReadPropertyMultipleObject {
    pub object_id: ObjectId, // e.g ObjectDevice:20088
    pub property_ids: Vec<PropertyId>,
}

impl ReadPropertyMultipleObject {
    pub fn new(object_id: ObjectId, property_ids: Vec<PropertyId>) -> Self {
        Self {
            object_id,
            property_ids,
        }
    }
}

impl ReadPropertyMultiple {
    pub fn new(objects: Vec<ReadPropertyMultipleObject>) -> Self {
        Self {
            objects,
            array_index: BACNET_ARRAY_ALL,
        }
    }

    pub fn encode(&self, buffer: &mut Buffer) {
        for object in &self.objects {
            // object_id
            encode_context_object_id(buffer, 0, &object.object_id);

            encode_opening_tag(buffer, 1);

            for property_id in &object.property_ids {
                // property_id
                encode_context_enumerated(buffer, 0, *property_id);

                // array_index
                if self.array_index != BACNET_ARRAY_ALL {
                    encode_context_unsigned(buffer, 1, self.array_index);
                }
            }

            encode_closing_tag(buffer, 1);
        }
    }

    pub fn decode(_reader: &mut Reader) -> Self {
        unimplemented!()
    }
}
