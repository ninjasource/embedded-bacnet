use crate::{
    application_protocol::primitives::data_value::ApplicationDataValue,
    common::{
        helper::{
            decode_context_enumerated, encode_context_enumerated, encode_context_object_id,
            encode_context_unsigned, get_tagged_body, Reader, Writer,
        },
        object_id::ObjectId,
        property_id::PropertyId,
        spec::BACNET_ARRAY_ALL,
        tag::{ApplicationTagNumber, Tag, TagNumber},
    },
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadPropertyValue<'a> {
    ObjectIdList(ObjectIdList<'a>),
    ApplicationDataValue(ApplicationDataValue<'a>),
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ObjectIdList<'a> {
    reader: Reader,
    buf: &'a [u8],
}

impl<'a> Iterator for ObjectIdList<'a> {
    type Item = ObjectId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.reader.eof() {
            None
        } else {
            let tag = Tag::decode(&mut self.reader, &self.buf);
            match tag.number {
                TagNumber::Application(ApplicationTagNumber::ObjectId) => {
                    // ok
                }
                x => panic!("Unexpected tag number: {:?}", x),
            }
            let object_id = ObjectId::decode(tag.value, &mut self.reader, &self.buf).unwrap();
            Some(object_id)
        }
    }
}

/*
impl ObjectIdList {
    pub fn decode_next(&self, reader: &mut Reader, buf: &[u8]) -> Option<ObjectId> {
        let tag = Tag::decode(reader, buf);
        if tag.number == TagNumber::ContextSpecificClosing(3) {
            // closing tag
            return None;
        }

        let object_id = ObjectId::decode(tag.value, reader, buf).unwrap();
        Some(object_id)
    }
}*/

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadPropertyAck<'a> {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
    pub property_value: ReadPropertyValue<'a>,
}

impl<'a> ReadPropertyAck<'a> {
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Self {
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

        //let tag = Tag::decode(reader, buf);
        let inner_buf = get_tagged_body(3, reader, buf);
        let mut inner_reader = Reader {
            index: 0,
            end: inner_buf.len(),
        };
        //assert_eq!(
        //    tag.number,
        //    TagNumber::ContextSpecificOpening(3),
        //    "expected opening tag"
        //);

        match property_id {
            PropertyId::PropObjectList => {
                let property_value = ReadPropertyValue::ObjectIdList(ObjectIdList {
                    reader: inner_reader,
                    buf: inner_buf,
                });

                return Self {
                    object_id,
                    property_id,
                    property_value,
                };
            }
            property_id => {
                let tag = Tag::decode(&mut inner_reader, buf);
                let value = ApplicationDataValue::decode(
                    &tag,
                    &object_id,
                    &property_id,
                    &mut inner_reader,
                    buf,
                );
                let property_value = ReadPropertyValue::ApplicationDataValue(value);

                //let tag = Tag::decode(reader, buf);
                //assert_eq!(
                //    tag.number,
                //    TagNumber::ContextSpecificClosing(3),
                //    "expected closing tag"
                //);

                return Self {
                    object_id,
                    property_id,
                    property_value,
                };
            }
        }
    }
}

#[derive(Debug)]
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
