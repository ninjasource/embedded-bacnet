use heapless::Vec;

use crate::common::{
    helper::{
        encode_closing_tag, encode_context_enumerated, encode_context_object_id,
        encode_context_unsigned, encode_opening_tag, Buffer, Reader,
    },
    object_id::ObjectId,
    property_id::PropertyId,
    spec::BACNET_ARRAY_ALL,
};

#[derive(Debug)]
pub struct ReadPropertyMultiple {
    pub object_id: ObjectId, // e.g ObjectDevice:20088
    pub property_ids: Vec<PropertyId, 512>,
    pub array_index: u32, // use BACNET_ARRAY_ALL for all
}

impl ReadPropertyMultiple {
    pub fn new(object_id: ObjectId, property_ids: Vec<PropertyId, 512>) -> Self {
        Self {
            object_id,
            property_ids,
            array_index: BACNET_ARRAY_ALL,
        }
    }

    pub fn encode(&self, buffer: &mut Buffer) {
        // object_id
        encode_context_object_id(buffer, 0, &self.object_id);

        encode_opening_tag(buffer, 1);

        for property_id in &self.property_ids {
            // property_id
            encode_context_enumerated(buffer, 0, *property_id);

            // array_index
            if self.array_index != BACNET_ARRAY_ALL {
                encode_context_unsigned(buffer, 1, self.array_index);
            }
        }

        encode_closing_tag(buffer, 1);
    }

    pub fn decode(_reader: &mut Reader) -> Self {
        unimplemented!()
    }
}
