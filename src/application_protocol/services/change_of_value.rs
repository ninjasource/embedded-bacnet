// change of value

use crate::{
    application_protocol::primitives::data_value::ApplicationDataValue,
    common::{
        error::Error,
        helper::{
            decode_unsigned, encode_context_bool, encode_context_object_id,
            encode_context_unsigned, Reader, Writer,
        },
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
        tag::{Tag, TagNumber},
    },
};

#[derive(Debug)]
pub struct CovNotification {
    pub process_id: u32,
    pub device_id: ObjectId,
    pub object_id: ObjectId,
    pub time_remaining_seconds: u32,
}

#[derive(Debug)]
pub struct PropertyResult<'a> {
    pub id: PropertyId,
    pub value: ApplicationDataValue<'a>,
}

impl CovNotification {
    const TAG_PROCESS_ID: u8 = 0;
    const TAG_DEVICE_ID: u8 = 1;
    const TAG_OBJECT_ID: u8 = 2;
    const TAG_LIFETIME: u8 = 3;
    const TAG_LIST_OF_VALUES: u8 = 4;

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        // parse a tag, starting from after the pdu type and service choice

        // process_id
        let tag = Tag::decode(reader, buf);
        if tag.number != TagNumber::ContextSpecific(Self::TAG_PROCESS_ID) {
            return Err(Error::InvalidValue(
                "expected process_id tag type for CovNotification",
            ));
        }
        let process_id = decode_unsigned(tag.value, reader, buf) as u32;

        // device_id
        let tag = Tag::decode(reader, buf);
        if tag.number != TagNumber::ContextSpecific(Self::TAG_DEVICE_ID) {
            return Err(Error::InvalidValue(
                "expected device_id tag type for CovNotification",
            ));
        }
        let device_id = ObjectId::decode(tag.value, reader, buf)?;
        if device_id.object_type != ObjectType::ObjectDevice {
            return Err(Error::InvalidValue(
                "expected device object type for CovNotification device_id field",
            ));
        }

        // object_id
        let tag = Tag::decode(reader, buf);
        if tag.number != TagNumber::ContextSpecific(Self::TAG_OBJECT_ID) {
            return Err(Error::InvalidValue(
                "expected object_id tag type for CovNotification",
            ));
        }
        let object_id = ObjectId::decode(tag.value, reader, buf)?;

        // lifetime
        let tag = Tag::decode(reader, buf);
        if tag.number != TagNumber::ContextSpecific(Self::TAG_LIFETIME) {
            return Err(Error::InvalidValue(
                "expected lifetime tag type for CovNotification",
            ));
        }
        let time_remaining_seconds = decode_unsigned(tag.value, reader, buf) as u32;

        // opening tag list of values
        let tag = Tag::decode(reader, buf);
        if tag.number != TagNumber::ContextSpecific(Self::TAG_LIST_OF_VALUES) {
            return Err(Error::InvalidValue(
                "expected list of values opening tag type for CovNotification",
            ));
        }

        Ok(Self {
            process_id,
            device_id,
            object_id,
            time_remaining_seconds,
        })
    }

    pub fn decode_next<'a>(
        &self,
        reader: &mut Reader,
        buf: &'a [u8],
    ) -> Option<PropertyResult<'a>> {
        // TODO: read list of values

        let tag = Tag::decode(reader, buf);
        if tag.number == TagNumber::ContextSpecific(4) {
            return None;
        }

        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(0),
            "invalid property id tag"
        );

        let property_id: PropertyId = (decode_unsigned(tag.value, reader, buf) as u32).into();

        let tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(2),
            "expected value opening tag"
        );

        let tag = Tag::decode(reader, buf);
        let value = ApplicationDataValue::decode(&tag, &self.object_id, &property_id, reader, buf);

        let tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(2),
            "expected value closing tag"
        );

        Some(PropertyResult {
            id: property_id,
            value,
        })
    }
}

#[derive(Debug)]
pub struct SubscribeCov {
    process_id: u32,
    object_id: ObjectId,
    issue_confirmed_notifications: bool,
    lifetime_seconds: u32, // zero for indefinite
}

impl SubscribeCov {
    const TAG_PROCESS_ID: u8 = 0;
    const TAG_OBJECT_ID: u8 = 1;
    const TAG_CONFIRMED: u8 = 2;
    const TAG_LIFETIME: u8 = 3;

    pub fn new(
        process_id: u32,
        object_id: ObjectId,
        issue_confirmed_notifications: bool,
        lifetime_seconds: u32,
    ) -> Self {
        Self {
            process_id,
            object_id,
            issue_confirmed_notifications,
            lifetime_seconds,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        // subscriber process_id
        encode_context_unsigned(writer, Self::TAG_PROCESS_ID, self.process_id);

        // object_id
        encode_context_object_id(writer, Self::TAG_OBJECT_ID, &self.object_id);

        // issue confirmed notifications
        encode_context_bool(
            writer,
            Self::TAG_CONFIRMED,
            self.issue_confirmed_notifications,
        );

        // lifetime of subscription
        encode_context_unsigned(writer, Self::TAG_LIFETIME, self.lifetime_seconds);
    }
}
