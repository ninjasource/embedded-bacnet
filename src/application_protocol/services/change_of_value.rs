// change of value

use crate::{
    application_protocol::primitives::data_value::ApplicationDataValue,
    common::{
        error::Error,
        helper::{
            decode_unsigned, encode_context_bool, encode_context_object_id,
            encode_context_unsigned, get_tagged_body,
        },
        io::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
        tag::{Tag, TagNumber},
    },
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CovNotification<'a> {
    pub process_id: u32,
    pub device_id: ObjectId,
    pub object_id: ObjectId,
    pub time_remaining_seconds: u32,
    reader: Reader,
    buf: &'a [u8],
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PropertyResult<'a> {
    pub id: PropertyId,
    pub value: ApplicationDataValue<'a>,
}

impl<'a> CovNotification<'a> {
    const TAG_PROCESS_ID: u8 = 0;
    const TAG_DEVICE_ID: u8 = 1;
    const TAG_OBJECT_ID: u8 = 2;
    const TAG_LIFETIME: u8 = 3;
    const TAG_LIST_OF_VALUES: u8 = 4;

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
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

        let (buf, tag_number) = get_tagged_body(reader, buf);
        if tag_number != Self::TAG_LIST_OF_VALUES {
            return Err(Error::InvalidValue(
                "expected list of values opening tag type for CovNotification",
            ));
        }

        let reader = Reader {
            index: 0,
            end: buf.len(),
        };

        Ok(Self {
            process_id,
            device_id,
            object_id,
            time_remaining_seconds,
            buf,
            reader,
        })
    }
}

impl<'a> Iterator for CovNotification<'a> {
    type Item = PropertyResult<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            return None;
        }

        let tag = Tag::decode(&mut self.reader, self.buf);

        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(0),
            "invalid property id tag"
        );

        let property_id: PropertyId =
            (decode_unsigned(tag.value, &mut self.reader, self.buf) as u32).into();

        let tag = Tag::decode(&mut self.reader, self.buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecificOpening(2),
            "expected value opening tag"
        );

        let tag = Tag::decode(&mut self.reader, self.buf);
        let value = ApplicationDataValue::decode(
            &tag,
            &self.object_id,
            &property_id,
            &mut self.reader,
            self.buf,
        );

        let tag = Tag::decode(&mut self.reader, self.buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecificClosing(2),
            "expected value closing tag"
        );

        Some(PropertyResult {
            id: property_id,
            value,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
