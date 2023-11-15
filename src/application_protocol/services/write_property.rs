use crate::{
    application_protocol::primitives::data_value::ApplicationDataValueWrite,
    common::{
        error::Error,
        helper::{
            decode_context_object_id, decode_context_property_id, decode_unsigned,
            encode_closing_tag, encode_context_enumerated, encode_context_object_id,
            encode_context_unsigned, encode_opening_tag,
        },
        io::{Reader, Writer},
        object_id::ObjectId,
        property_id::PropertyId,
        spec::BACNET_ARRAY_ALL,
        tag::{Tag, TagNumber},
    },
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WriteProperty<'a> {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
    pub priority: Option<u8>,
    pub array_index: Option<u32>,
    pub value: ApplicationDataValueWrite<'a>,
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

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let object_id = decode_context_object_id(reader, buf, Self::TAG_OBJECT_ID)?;
        let property_id = decode_context_property_id(
            reader,
            buf,
            Self::TAG_PROPERTY_ID,
            "WriteProperty decode property_id",
        )?;

        // array_index
        let mut tag = Tag::decode(reader, buf);
        let mut array_index = None;
        if let TagNumber::ContextSpecific(Self::TAG_ARRAY_INDEX) = tag.number {
            let array_index_tmp = decode_unsigned(tag.value, reader, buf) as u32;
            if array_index_tmp != BACNET_ARRAY_ALL {
                array_index = Some(array_index_tmp)
            }

            // read another tag
            tag = Tag::decode(reader, buf);
        }

        assert_eq!(
            tag.number,
            TagNumber::ContextSpecificOpening(Self::TAG_VALUE)
        );
        let value = ApplicationDataValueWrite::decode(&object_id, &property_id, reader, buf)?;
        tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecificClosing(Self::TAG_VALUE)
        );

        let tag = Tag::decode(reader, buf);
        assert_eq!(tag.number, TagNumber::ContextSpecific(Self::TAG_PRIORITY));
        let priority = tag.value as u8;
        let priority = if priority == Self::LOWEST_PRIORITY {
            None
        } else {
            Some(priority)
        };

        Ok(Self {
            object_id,
            property_id,
            array_index,
            value,
            priority,
        })
    }

    pub fn encode(&self, writer: &mut Writer) {
        // object_id
        encode_context_object_id(writer, Self::TAG_OBJECT_ID, &self.object_id);

        // property_id
        encode_context_enumerated(writer, Self::TAG_PROPERTY_ID, &self.property_id);

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
