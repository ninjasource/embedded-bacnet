use crate::{
    application_protocol::{
        confirmed::{ComplexAck, ComplexAckService, ConfirmedServiceChoice},
        primitives::data_value::ApplicationDataValue,
    },
    common::{
        error::Error,
        helper::{
            decode_context_object_id, decode_context_property_id, encode_closing_tag,
            encode_context_enumerated, encode_context_object_id, encode_context_unsigned,
            encode_opening_tag, get_tagged_body_for_tag,
        },
        io::{Reader, Writer},
        object_id::ObjectId,
        property_id::PropertyId,
        spec::BACNET_ARRAY_ALL,
        tag::{ApplicationTagNumber, Tag, TagNumber},
    },
    network_protocol::data_link::DataLink,
};

#[cfg(feature = "alloc")]
use {
    crate::common::spooky::Phantom, alloc::vec::Vec, bacnet_macros::remove_lifetimes_from_fn_args,
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadPropertyValue<'a> {
    ObjectIdList(ObjectIdList<'a>),
    ApplicationDataValue(ApplicationDataValue<'a>),
}

#[cfg(not(feature = "alloc"))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ObjectIdList<'a> {
    object_ids: &'a [ObjectId],
    buf: &'a [u8],
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ObjectIdList<'a> {
    pub object_ids: Vec<ObjectId>,
    _phantom: &'a Phantom,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ObjectIdIter<'a> {
    reader: Reader,
    buf: &'a [u8],
}

impl<'a> ObjectIdList<'a> {
    #[cfg(not(feature = "alloc"))]
    pub fn new(object_ids: &'a [ObjectId]) -> Self {
        Self {
            object_ids,
            buf: &[],
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn new_from_buf(buf: &'a [u8]) -> Self {
        Self {
            object_ids: &[],
            buf,
        }
    }

    #[cfg(feature = "alloc")]
    pub fn new(object_ids: Vec<ObjectId>) -> Self {
        use crate::common::spooky::PHANTOM;

        Self {
            object_ids,
            _phantom: &PHANTOM,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        for object_id in self.object_ids.iter() {
            Tag::new(
                TagNumber::Application(ApplicationTagNumber::ObjectId),
                ObjectId::LEN,
            )
            .encode(writer);
            object_id.encode(writer);
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn decode(_reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        Ok(Self::new_from_buf(buf))
    }

    #[cfg(feature = "alloc")]
    pub fn decode(_reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let decoder = ObjectIdIter::new(buf);
        let object_ids: Result<Vec<ObjectId>, Error> = decoder.into_iter().collect();
        Ok(Self::new(object_ids?))
    }
}

impl<'a> ObjectIdIter<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            reader: Reader::new_with_len(buf.len()),
            buf,
        }
    }

    fn next_internal(&mut self) -> Result<ObjectId, Error> {
        let tag = Tag::decode_expected(
            &mut self.reader,
            self.buf,
            TagNumber::Application(ApplicationTagNumber::ObjectId),
            "ObjectIdList nex",
        )?;

        ObjectId::decode(tag.value, &mut self.reader, self.buf)
    }
}

#[cfg(not(feature = "alloc"))]
impl<'a> IntoIterator for &'_ ObjectIdList<'a> {
    type Item = Result<ObjectId, Error>;
    type IntoIter = ObjectIdIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ObjectIdIter::new(self.buf)
    }
}

impl<'a> Iterator for ObjectIdIter<'a> {
    type Item = Result<ObjectId, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            None
        } else {
            Some(self.next_internal())
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadPropertyAck<'a> {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
    pub property_value: ReadPropertyValue<'a>,
}

impl<'a> TryFrom<DataLink<'a>> for ReadPropertyAck<'a> {
    type Error = Error;

    fn try_from(value: DataLink<'a>) -> Result<Self, Self::Error> {
        let ack: ComplexAck = value.try_into()?;
        match ack.service {
            ComplexAckService::ReadProperty(ack) => Ok(ack),
            _ => Err(Error::ConvertDataLink(
                "apdu message is not a ComplexAckService ReadPropertyAck",
            )),
        }
    }
}

impl<'a> ReadPropertyAck<'a> {
    pub fn encode(&self, writer: &mut Writer) {
        writer.push(ConfirmedServiceChoice::ReadProperty as u8);
        encode_context_object_id(writer, 0, &self.object_id);
        encode_context_enumerated(writer, 1, &self.property_id);
        encode_opening_tag(writer, 3);
        match &self.property_value {
            ReadPropertyValue::ApplicationDataValue(value) => {
                value.encode(writer);
            }
            ReadPropertyValue::ObjectIdList(value) => {
                value.encode(writer);
            }
        }
        encode_closing_tag(writer, 3);
    }

    #[cfg_attr(feature = "alloc", remove_lifetimes_from_fn_args)]
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let object_id =
            decode_context_object_id(reader, buf, 0, "ReadPropertyAck decode object_id")?;
        let property_id =
            decode_context_property_id(reader, buf, 1, "ReadPropertyAck decode property_id")?;

        let buf = get_tagged_body_for_tag(reader, buf, 3, "ReadPropertyAck decode data values")?;
        let mut reader = Reader::new_with_len(buf.len());

        match property_id {
            PropertyId::PropObjectList => {
                let object_ids = ObjectIdList::decode(&mut reader, buf)?;
                let property_value = ReadPropertyValue::ObjectIdList(object_ids);

                Ok(Self {
                    object_id,
                    property_id,
                    property_value,
                })
            }
            property_id => {
                let tag = Tag::decode(&mut reader, buf)?;
                let value =
                    ApplicationDataValue::decode(&tag, &object_id, &property_id, &mut reader, buf)?;
                let property_value = ReadPropertyValue::ApplicationDataValue(value);

                Ok(Self {
                    object_id,
                    property_id,
                    property_value,
                })
            }
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
        encode_context_enumerated(writer, 1, &self.property_id);

        // array_index
        if self.array_index != BACNET_ARRAY_ALL {
            encode_context_unsigned(writer, 2, self.array_index);
        }
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        // object_id
        let object_id = decode_context_object_id(reader, buf, 0, "ReadProperty decode object_id")?;

        // property_id
        let property_id =
            decode_context_property_id(reader, buf, 1, "ReadProperty decode property_id")?;

        Ok(Self {
            object_id,
            property_id,
            array_index: BACNET_ARRAY_ALL,
        })
    }
}
