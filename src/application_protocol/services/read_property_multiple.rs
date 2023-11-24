use core::fmt::Display;

use crate::{
    application_protocol::{
        confirmed::{ComplexAck, ComplexAckService, ConfirmedServiceChoice},
        primitives::data_value::ApplicationDataValue,
    },
    common::{
        daily_schedule::WeeklySchedule,
        error::Error,
        helper::{
            decode_context_object_id, decode_context_property_id, decode_unsigned,
            encode_closing_tag, encode_context_enumerated, encode_context_object_id,
            encode_context_unsigned, encode_opening_tag, get_tagged_body, get_tagged_body_for_tag,
        },
        io::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
        spec::{ErrorClass, ErrorCode, BACNET_ARRAY_ALL},
        tag::{ApplicationTagNumber, Tag, TagNumber},
    },
    network_protocol::data_link::DataLink,
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadPropertyMultipleAck<'a> {
    pub objects_with_results: &'a [ObjectWithResults<'a>],
    buf: &'a [u8],
}

impl<'a> IntoIterator for &'_ ReadPropertyMultipleAck<'a> {
    type Item = Result<ObjectWithResults<'a>, Error>;

    type IntoIter = ObjectWithResultsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ObjectWithResultsIter {
            buf: self.buf,
            reader: Reader::new_with_len(self.buf.len()),
        }
    }
}

impl<'a> TryFrom<DataLink<'a>> for ReadPropertyMultipleAck<'a> {
    type Error = Error;

    fn try_from(value: DataLink<'a>) -> Result<Self, Self::Error> {
        let ack: ComplexAck = value.try_into()?;
        match ack.service {
            ComplexAckService::ReadPropertyMultiple(ack) => Ok(ack),
            _ => Err(Error::ConvertDataLink(
                "apdu message is not a ComplexAckService ReadPropertyMultipleAck",
            )),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ObjectWithResults<'a> {
    pub object_id: ObjectId,
    pub property_results: PropertyResultList<'a>,
}

impl<'a> ObjectWithResults<'a> {
    pub fn encode(&self, writer: &mut Writer) {
        encode_context_object_id(writer, 0, &self.object_id);
        encode_opening_tag(writer, 1);
        self.property_results.encode(writer);
        encode_closing_tag(writer, 1);
    }

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let object_id =
            decode_context_object_id(reader, buf, 0, "ObjectWithResults decode object_id")?;
        let buf =
            get_tagged_body_for_tag(reader, buf, 1, "ObjectWithResults decode list of results")?;

        let property_results = PropertyResultList {
            object_id,
            buf,
            property_results: &[],
        };

        Ok(ObjectWithResults {
            object_id,
            property_results,
        })
    }
}

impl<'a> IntoIterator for &'_ PropertyResultList<'a> {
    type Item = Result<PropertyResult<'a>, Error>;
    type IntoIter = PropertyResultIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PropertyResultIter {
            buf: self.buf,
            reader: Reader::new_with_len(self.buf.len()),
            object_id: self.object_id,
        }
    }
}

impl<'a> Iterator for PropertyResultIter<'a> {
    type Item = Result<PropertyResult<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            return None;
        }

        Some(PropertyResult::decode(
            &mut self.reader,
            self.buf,
            &self.object_id,
        ))
    }
}

fn read_error(reader: &mut Reader, buf: &[u8]) -> Result<PropertyAccessError, Error> {
    // error class enumerated
    let tag = Tag::decode_expected(
        reader,
        buf,
        TagNumber::Application(ApplicationTagNumber::Enumerated),
        "read_error error_class",
    )?;
    let value = decode_unsigned(tag.value, reader, buf)? as u32;
    let error_class = value
        .try_into()
        .map_err(|x| Error::InvalidVariant(("ErrorClass", x)))?;

    // error code enumerated
    let tag = Tag::decode_expected(
        reader,
        buf,
        TagNumber::Application(ApplicationTagNumber::Enumerated),
        "read_error error code",
    )?;
    let value = decode_unsigned(tag.value, reader, buf)? as u32;
    let error_code = value
        .try_into()
        .map_err(|x| Error::InvalidVariant(("ErrorCode", x)))?;

    Ok(PropertyAccessError {
        error_class,
        error_code,
    })
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PropertyResultList<'a> {
    pub property_results: &'a [PropertyResult<'a>],
    object_id: ObjectId,
    buf: &'a [u8],
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PropertyResultIter<'a> {
    object_id: ObjectId,
    reader: Reader,
    buf: &'a [u8],
}

impl<'a> PropertyResultList<'a> {
    pub fn new(property_results: &'a [PropertyResult<'a>]) -> Self {
        Self {
            property_results,
            object_id: ObjectId::new(ObjectType::Invalid, 0),
            buf: &[],
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        for item in self.property_results {
            item.encode(writer);
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PropertyResult<'a> {
    pub id: PropertyId,
    pub value: PropertyValue<'a>,
}

impl<'a> PropertyResult<'a> {
    const PROPERTY_ID_TAG: u8 = 2;

    pub fn encode(&self, writer: &mut Writer) {
        encode_context_unsigned(writer, Self::PROPERTY_ID_TAG, self.id as u32);
        self.value.encode(writer);
    }

    pub fn decode(reader: &mut Reader, buf: &'a [u8], object_id: &ObjectId) -> Result<Self, Error> {
        let property_id = decode_context_property_id(
            reader,
            buf,
            Self::PROPERTY_ID_TAG,
            "PropertyResultList next property_id",
        )?;

        let value = PropertyValue::decode(reader, buf, object_id, &property_id)?;

        Ok(PropertyResult {
            id: property_id,
            value,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PropertyValue<'a> {
    PropValue(ApplicationDataValue<'a>),
    PropError(PropertyAccessError),
    // TODO: figure out if we need these
    PropDescription(&'a str),
    PropObjectName(&'a str),
}

impl<'a> PropertyValue<'a> {
    const PROPERTY_VALUE_TAG: u8 = 4;
    const PROPERTY_ERROR_TAG: u8 = 5;

    pub fn encode(&self, writer: &mut Writer) {
        match self {
            Self::PropValue(val) => {
                encode_opening_tag(writer, Self::PROPERTY_VALUE_TAG);
                val.encode(writer);
                encode_closing_tag(writer, Self::PROPERTY_VALUE_TAG);
            }
            Self::PropError(_) => todo!(),
            Self::PropObjectName(_) => todo!(),
            Self::PropDescription(_) => todo!(),
        }
    }

    pub fn decode(
        reader: &mut Reader,
        buf: &'a [u8],
        object_id: &ObjectId,
        property_id: &PropertyId,
    ) -> Result<Self, Error> {
        let (buf, tag_number) = get_tagged_body(reader, buf)?;
        let mut reader = Reader {
            index: 0,
            end: buf.len(),
        };

        let property_value = match tag_number {
            Self::PROPERTY_VALUE_TAG => {
                match &property_id {
                    PropertyId::PropEventTimeStamps => {
                        // ignore for now
                        PropertyValue::PropValue(ApplicationDataValue::Boolean(false))
                    }
                    PropertyId::PropWeeklySchedule => {
                        let weekly_schedule = WeeklySchedule::decode(&mut reader, buf)?;
                        PropertyValue::PropValue(ApplicationDataValue::WeeklySchedule(
                            weekly_schedule,
                        ))
                    }
                    property_id => {
                        let tag = Tag::decode(&mut reader, buf)?;
                        let value = ApplicationDataValue::decode(
                            &tag,
                            object_id,
                            property_id,
                            &mut reader,
                            buf,
                        )?;
                        PropertyValue::PropValue(value)
                    }
                }
            }
            Self::PROPERTY_ERROR_TAG => {
                // property read error
                let error = read_error(&mut reader, buf)?;
                PropertyValue::PropError(error)
            }
            x => {
                return Err(Error::TagNotSupported((
                    "PropertyResultList next",
                    TagNumber::ContextSpecificOpening(x),
                )));
            }
        };

        Ok(property_value)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PropertyAccessError {
    pub error_class: ErrorClass,
    pub error_code: ErrorCode,
}

impl<'a> Display for PropertyValue<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self {
            Self::PropValue(x) => write!(f, "{}", x),
            _ => write!(f, "property value unprintable",),
        }
    }
}

impl<'a> ReadPropertyMultipleAck<'a> {
    pub fn new(objects_with_results: &'a [ObjectWithResults<'a>]) -> Self {
        Self {
            objects_with_results,
            buf: &[],
        }
    }

    pub fn new_from_buf(buf: &'a [u8]) -> Self {
        Self {
            buf,
            objects_with_results: &[],
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        writer.push(ConfirmedServiceChoice::ReadPropMultiple as u8);
        for item in self.objects_with_results {
            item.encode(writer);
        }
    }
}

pub struct ObjectWithResultsIter<'a> {
    buf: &'a [u8],
    reader: Reader,
}

impl<'a> Iterator for ObjectWithResultsIter<'a> {
    type Item = Result<ObjectWithResults<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            return None;
        }

        let object_with_results = ObjectWithResults::decode(&mut self.reader, self.buf);
        Some(object_with_results)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadPropertyMultiple<'a> {
    _array_index: u32, // use BACNET_ARRAY_ALL for all
    objects: &'a [ReadPropertyMultipleObject<'a>],
    buf: &'a [u8],
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PropertyIdList<'a> {
    pub property_ids: &'a [PropertyId],
    buf: &'a [u8],
}

impl<'a> IntoIterator for &'_ PropertyIdList<'a> {
    type Item = Result<PropertyId, Error>;

    type IntoIter = PropertyIdIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PropertyIdIter {
            buf: self.buf,
            reader: Reader::new_with_len(self.buf.len()),
        }
    }
}

pub struct PropertyIdIter<'a> {
    reader: Reader,
    buf: &'a [u8],
}

impl<'a> Iterator for PropertyIdIter<'a> {
    type Item = Result<PropertyId, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            None
        } else {
            match decode_context_property_id(
                &mut self.reader,
                self.buf,
                0,
                "PropertyIdList next property_id",
            ) {
                Ok(property_id) => Some(Ok(property_id)),
                Err(e) => Some(Err(e)),
            }
        }
    }
}

impl<'a> PropertyIdList<'a> {
    pub fn new(property_ids: &'a [PropertyId]) -> Self {
        Self {
            property_ids,
            buf: &[],
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        encode_opening_tag(writer, 1);

        for property_id in self.property_ids {
            // property_id
            encode_context_enumerated(writer, 0, property_id);

            // array_index
            //if self.array_index != BACNET_ARRAY_ALL {
            //    encode_context_unsigned(writer, 1, self.array_index);
            //}
        }

        encode_closing_tag(writer, 1);
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadPropertyMultipleObject<'a> {
    pub object_id: ObjectId, // e.g ObjectDevice:20088
    pub property_ids: PropertyIdList<'a>,
}

impl<'a> ReadPropertyMultipleObject<'a> {
    pub fn new(object_id: ObjectId, property_ids: &'a [PropertyId]) -> Self {
        let property_ids = PropertyIdList::new(property_ids);
        Self {
            object_id,
            property_ids,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        // object_id
        encode_context_object_id(writer, 0, &self.object_id);

        encode_opening_tag(writer, 1);

        for property_id in self.property_ids.property_ids {
            // property_id
            encode_context_enumerated(writer, 0, property_id);

            // array_index
            //if self.array_index != BACNET_ARRAY_ALL {
            //    encode_context_unsigned(writer, 1, self.array_index);
            //}
        }

        encode_closing_tag(writer, 1);
    }

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let object_id =
            decode_context_object_id(reader, buf, 0, "ReadPropertyMultiple next object_id")?;

        let buf =
            get_tagged_body_for_tag(reader, buf, 1, "ReadPropertyMultiple next list of results")?;
        let property_ids = PropertyIdList {
            property_ids: &[],
            buf,
        };

        Ok(ReadPropertyMultipleObject {
            object_id,
            property_ids,
        })
    }
}

impl<'a> ReadPropertyMultiple<'a> {
    pub fn new(objects: &'a [ReadPropertyMultipleObject]) -> Self {
        Self {
            objects,
            _array_index: BACNET_ARRAY_ALL,
            buf: &[],
        }
    }

    pub fn new_from_buf(buf: &'a [u8]) -> Self {
        Self {
            objects: &[],
            _array_index: BACNET_ARRAY_ALL,
            buf,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        for object in self.objects {
            object.encode(writer)
        }
    }

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Self {
        let buf = &buf[reader.index..reader.end];
        Self {
            buf,
            _array_index: BACNET_ARRAY_ALL,
            objects: &[],
        }
    }
}

impl<'a> IntoIterator for &'_ ReadPropertyMultiple<'a> {
    type Item = Result<ReadPropertyMultipleObject<'a>, Error>;

    type IntoIter = ReadPropertyMultipleIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ReadPropertyMultipleIter {
            buf: self.buf,
            reader: Reader::new_with_len(self.buf.len()),
        }
    }
}

pub struct ReadPropertyMultipleIter<'a> {
    buf: &'a [u8],
    reader: Reader,
}

impl<'a> Iterator for ReadPropertyMultipleIter<'a> {
    type Item = Result<ReadPropertyMultipleObject<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            return None;
        }

        let object_with_property_ids =
            ReadPropertyMultipleObject::decode(&mut self.reader, self.buf);
        Some(object_with_property_ids)
    }
}
