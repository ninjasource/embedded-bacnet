// change of value

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{
    application_protocol::primitives::data_value::ApplicationDataValue,
    common::{
        error::Error,
        helper::{
            decode_unsigned, encode_context_bool, encode_context_object_id,
            encode_context_unsigned, get_tagged_body_for_tag,
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
    pub values: CovNotificationValues<'a>,
}

#[cfg(not(feature = "alloc"))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CovNotificationValues<'a> {
    _property_results: &'a [PropertyResult<'a>], // use this when encoding is implemented
    object_id: ObjectId,
    buf: &'a [u8],
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CovNotificationValues<'a> {
    pub property_results: Vec<PropertyResult<'a>>, // use this when encoding is implemented
    pub object_id: ObjectId,
}

impl<'a> CovNotificationValues<'a> {
    #[cfg(not(feature = "alloc"))]
    pub fn decode(_reader: &mut Reader, buf: &'a [u8], object_id: ObjectId) -> Result<Self, Error> {
        Ok(CovNotificationValues {
            buf,
            _property_results: &[],
            object_id,
        })
    }

    #[cfg(feature = "alloc")]
    pub fn decode(reader: &mut Reader, buf: &[u8], object_id: ObjectId) -> Result<Self, Error> {
        let mut property_results = Vec::new();

        while !reader.eof() {
            let result = PropertyResult::decode(reader, buf, &object_id)?;
            property_results.push(result);
        }

        Ok(Self {
            object_id,
            property_results,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PropertyResult<'a> {
    pub id: PropertyId,
    pub value: ApplicationDataValue<'a>,
}

impl<'a> PropertyResult<'a> {
    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(reader: &mut Reader, buf: &'a [u8], object_id: &ObjectId) -> Result<Self, Error> {
        // property id
        let tag = Tag::decode_expected(
            reader,
            buf,
            TagNumber::ContextSpecific(0),
            "CovNotification next property_id",
        )?;
        let property_id: PropertyId = (decode_unsigned(tag.value, reader, buf)? as u32).into();

        // value
        Tag::decode_expected(
            reader,
            buf,
            TagNumber::ContextSpecificOpening(2),
            "CovNotification next expected value opening tag",
        )?;
        let tag = Tag::decode(reader, buf)?;
        let value = ApplicationDataValue::decode(&tag, object_id, &property_id, reader, buf)?;
        Tag::decode_expected(
            reader,
            buf,
            TagNumber::ContextSpecificClosing(2),
            "CovNotification next expected value closing tag",
        )?;

        Ok(PropertyResult {
            id: property_id,
            value,
        })
    }
}

impl<'a> CovNotification<'a> {
    const TAG_PROCESS_ID: u8 = 0;
    const TAG_DEVICE_ID: u8 = 1;
    const TAG_OBJECT_ID: u8 = 2;
    const TAG_LIFETIME: u8 = 3;
    const TAG_LIST_OF_VALUES: u8 = 4;

    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        // parse a tag, starting from after the pdu type and service choice

        // process_id
        let tag = Tag::decode_expected(
            reader,
            buf,
            TagNumber::ContextSpecific(Self::TAG_PROCESS_ID),
            "CovNotification process_id",
        )?;
        let process_id = decode_unsigned(tag.value, reader, buf)? as u32;

        // device_id
        let tag = Tag::decode_expected(
            reader,
            buf,
            TagNumber::ContextSpecific(Self::TAG_DEVICE_ID),
            "CovNotification device_id tag",
        )?;
        let device_id = ObjectId::decode(tag.value, reader, buf)?;
        if device_id.object_type != ObjectType::ObjectDevice {
            return Err(Error::InvalidValue(
                "expected device object type for CovNotification device_id field",
            ));
        }

        // object_id
        let tag = Tag::decode_expected(
            reader,
            buf,
            TagNumber::ContextSpecific(Self::TAG_OBJECT_ID),
            "CovNotification object_id",
        )?;
        let object_id = ObjectId::decode(tag.value, reader, buf)?;

        // lifetime
        let tag = Tag::decode_expected(
            reader,
            buf,
            TagNumber::ContextSpecific(Self::TAG_LIFETIME),
            "CovNotification lifetime",
        )?;
        let time_remaining_seconds = decode_unsigned(tag.value, reader, buf)? as u32;

        // values
        let inner_buf = get_tagged_body_for_tag(
            reader,
            buf,
            Self::TAG_LIST_OF_VALUES,
            "CovNotification decode list of values",
        )?;
        let mut inner_reader = Reader::new_with_len(inner_buf.len());
        let values = CovNotificationValues::decode(&mut inner_reader, inner_buf, object_id)?;

        Ok(Self {
            process_id,
            device_id,
            object_id,
            time_remaining_seconds,
            values,
        })
    }
}

#[cfg(not(feature = "alloc"))]
impl<'a> IntoIterator for &'_ CovNotificationValues<'a> {
    type Item = Result<PropertyResult<'a>, Error>;
    type IntoIter = CovNotificationIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        CovNotificationIter {
            buf: self.buf,
            reader: Reader::new_with_len(self.buf.len()),
            object_id: self.object_id,
        }
    }
}

pub struct CovNotificationIter<'a> {
    object_id: ObjectId,
    reader: Reader,
    buf: &'a [u8],
}

impl<'a> Iterator for CovNotificationIter<'a> {
    type Item = Result<PropertyResult<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            return None;
        }

        let result = PropertyResult::decode(&mut self.reader, self.buf, &self.object_id);
        Some(result)
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
