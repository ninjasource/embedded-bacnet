use crate::{
    application_protocol::primitives::data_value::ApplicationDataValueWrite,
    common::{
        helper::{
            encode_closing_tag, encode_context_enumerated, encode_context_object_id,
            encode_context_unsigned, encode_opening_tag, Writer,
        },
        object_id::ObjectId,
        property_id::PropertyId,
    },
};

#[derive(Debug)]
pub struct WriteProperty<'a> {
    object_id: ObjectId,
    property_id: PropertyId,
    priority: Option<u8>,
    array_index: Option<u32>,
    value: ApplicationDataValueWrite<'a>,
}

impl<'a> WriteProperty<'a> {
    const TAG_OBJECT_ID: u8 = 0;
    const TAG_PROPERTY_ID: u8 = 1;
    const TAG_ARRAY_INDEX: u8 = 2;
    const TAG_VALUE: u8 = 3;
    const TAG_PRIORITY: u8 = 4;
    const LOWEST_PRIORITY: u8 = 16;

    pub fn new(
        object_id: ObjectId,
        property_id: PropertyId,
        priority: Option<u8>,
        array_index: Option<u32>,
        value: ApplicationDataValueWrite<'a>,
    ) -> Self {
        Self {
            object_id,
            property_id,
            priority,
            array_index,
            value,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        // object_id
        encode_context_object_id(writer, Self::TAG_OBJECT_ID, &self.object_id);

        // property_id
        encode_context_enumerated(writer, Self::TAG_PROPERTY_ID, self.property_id);

        // array_index
        if let Some(array_index) = self.array_index {
            encode_context_unsigned(writer, Self::TAG_ARRAY_INDEX, array_index);
        }

        // value
        encode_opening_tag(writer, Self::TAG_VALUE);
        self.value.encode(writer);
        encode_closing_tag(writer, Self::TAG_VALUE);

        // priority 0-16 (16 being lowest priority)
        let priority = self
            .priority
            .unwrap_or(Self::LOWEST_PRIORITY)
            .min(Self::LOWEST_PRIORITY) as u32;
        encode_context_unsigned(writer, Self::TAG_PRIORITY, priority);
    }
}
