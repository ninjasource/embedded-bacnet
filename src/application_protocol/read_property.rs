use crate::common::{
    helper::{
        decode_context_enumerated, encode_context_enumerated, encode_context_object_id,
        encode_context_unsigned, Reader, Writer,
    },
    object_id::ObjectId,
    property_id::PropertyId,
    spec::BACNET_ARRAY_ALL,
    tag::{Tag, TagNumber},
};

#[derive(Debug)]
pub struct ReadPropertyAck {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
}

impl ReadPropertyAck {
    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Self {
        let tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(0),
            "invalid object id tag"
        );
        let object_id = ObjectId::decode(tag.value, reader, buf).unwrap();
        let (tag, property_id) = decode_context_enumerated(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(1),
            "invalid property id tag"
        );

        match property_id {
            PropertyId::PropObjectList => {
                let tag = Tag::decode(reader, buf);
                assert_eq!(
                    tag.number,
                    TagNumber::ContextSpecific(3),
                    "expected opening tag"
                );

                return Self {
                    object_id,
                    property_id,
                };
            }
            _ => unimplemented!(),
        }
    }

    pub fn decode_next(&self, reader: &mut Reader, buf: &[u8]) -> Option<ObjectId> {
        let tag = Tag::decode(reader, buf);
        if tag.number == TagNumber::ContextSpecific(3) {
            // closing tag
            return None;
        }

        let object_id = ObjectId::decode(tag.value, reader, buf).unwrap();
        Some(object_id)
    }
}

#[derive(Debug)]
pub struct ReadProperty {
    pub object_id: ObjectId,     // e.g ObjectDevice:20088
    pub property_id: PropertyId, // e.g. PropObjectList
    pub array_index: u32,        // use BACNET_ARRAY_ALL for all
}

impl ReadProperty {
    pub fn new(object_id: ObjectId, property_id: PropertyId) -> Self {
        Self {
            object_id,
            property_id,
            array_index: BACNET_ARRAY_ALL,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        // object_id
        encode_context_object_id(writer, 0, &self.object_id);

        // property_id
        encode_context_enumerated(writer, 1, self.property_id);

        // array_index
        if self.array_index != BACNET_ARRAY_ALL {
            encode_context_unsigned(writer, 2, self.array_index);
        }
    }

    pub fn decode(_reader: &mut Reader) -> Self {
        unimplemented!()
    }
}
