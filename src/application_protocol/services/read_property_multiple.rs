use core::fmt::Display;

use crate::{
    application_protocol::primitives::data_value::ApplicationDataValue,
    common::{
        daily_schedule::WeeklySchedule,
        helper::{
            decode_unsigned, encode_closing_tag, encode_context_enumerated,
            encode_context_object_id, encode_context_unsigned, encode_opening_tag, get_tagged_body,
            Reader, Writer,
        },
        object_id::ObjectId,
        property_id::PropertyId,
        spec::BACNET_ARRAY_ALL,
        tag::{Tag, TagNumber},
    },
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadPropertyMultipleAck {}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ObjectWithResults {
    pub object_id: ObjectId,
}

impl ObjectWithResults {
    pub fn decode_next<'a>(
        &self,
        reader: &mut Reader,
        buf: &'a [u8],
    ) -> Option<PropertyResult<'a>> {
        let tag = Tag::decode(reader, buf);
        if tag.number == TagNumber::ContextSpecificClosing(1) {
            // closing tag
            return None;
        }

        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(2),
            "expected property identifier tag"
        );
        let property_id: PropertyId = (decode_unsigned(tag.value, reader, buf) as u32).into();

        //let tag = Tag::decode(reader, buf);
        let inner_buf = get_tagged_body(4, reader, buf);
        let mut inner_reader = Reader {
            index: 0,
            end: inner_buf.len(),
        };

        // let tag_number = match tag.number {
        //     TagNumber::ContextSpecificOpening(x) => x,
        //     x => panic!("Expected opening tag but got: {:?}", x),
        // };

        //assert_eq!(
        //    tag.number,
        //    TagNumber::ContextSpecificOpening(4),
        //    "expected opening tag"
        //);

        let property_value = match property_id {
            PropertyId::PropEventTimeStamps => {
                // ignore for now
                PropertyValue::PropValue(ApplicationDataValue::Boolean(false))
                /*
                // hack to read past complicated timestamps
                loop {
                    let byte = reader.read_byte(buf);
                    // read until we get to the closing tag
                    if byte == 0x4f {
                        break PropertyValue::PropValue(ApplicationDataValue::Boolean(false));
                    }
                }*/
            }
            PropertyId::PropWeeklySchedule => {
                let weekly_schedule = WeeklySchedule::new(&mut inner_reader, inner_buf);
                //let weekly_schedule = WeeklySchedule::new(reader, buf);
                PropertyValue::PropValue(ApplicationDataValue::WeeklySchedule(weekly_schedule))
            }
            property_id => {
                let tag = Tag::decode(&mut inner_reader, inner_buf);
                let value = ApplicationDataValue::decode(
                    &tag,
                    &self.object_id,
                    &property_id,
                    &mut inner_reader,
                    inner_buf,
                );

                /*
                                let tag = Tag::decode(reader, buf);
                                let value =
                                    ApplicationDataValue::decode(&tag, &self.object_id, &property_id, reader, buf);
                */
                let property_value = PropertyValue::PropValue(value);

                // let tag = Tag::decode(reader, buf);
                // assert_eq!(
                //     tag.number,
                //     TagNumber::ContextSpecificClosing(4),
                //     "expected closing tag"
                // );

                property_value
            }
        };

        let property_result = PropertyResult {
            id: property_id,
            value: property_value,
        };

        Some(property_result)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PropertyResult<'a> {
    pub id: PropertyId,
    pub value: PropertyValue<'a>,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PropertyValue<'a> {
    PropValue(ApplicationDataValue<'a>),
    PropDescription(&'a str),
    PropObjectName(&'a str),
}

impl<'a> Display for PropertyValue<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self {
            Self::PropValue(x) => write!(f, "{}", x),
            _ => write!(f, "property value unprintable",),
        }
    }
}

impl ReadPropertyMultipleAck {
    pub fn decode_next(&self, reader: &mut Reader, buf: &[u8]) -> Option<ObjectWithResults> {
        if reader.eof() {
            return None;
        }

        let tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecific(0),
            "expected object_id tag"
        );
        let object_id = ObjectId::decode(tag.value, reader, buf).unwrap();

        let tag = Tag::decode(reader, buf);
        assert_eq!(
            tag.number,
            TagNumber::ContextSpecificOpening(1),
            "expected list of results opening tag"
        );

        let object_with_results = ObjectWithResults { object_id };
        Some(object_with_results)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadPropertyMultiple<'a> {
    pub array_index: u32, // use BACNET_ARRAY_ALL for all
    pub objects: &'a [ReadPropertyMultipleObject<'a>],
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReadPropertyMultipleObject<'a> {
    pub object_id: ObjectId, // e.g ObjectDevice:20088
    pub property_ids: &'a [PropertyId],
}

impl<'a> ReadPropertyMultipleObject<'a> {
    pub fn new(object_id: ObjectId, property_ids: &'a [PropertyId]) -> Self {
        Self {
            object_id,
            property_ids,
        }
    }
}

impl<'a> ReadPropertyMultiple<'a> {
    pub fn new(objects: &'a [ReadPropertyMultipleObject]) -> Self {
        Self {
            objects,
            array_index: BACNET_ARRAY_ALL,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        for object in self.objects {
            // object_id
            encode_context_object_id(writer, 0, &object.object_id);

            encode_opening_tag(writer, 1);

            for property_id in object.property_ids {
                // property_id
                encode_context_enumerated(writer, 0, *property_id);

                // array_index
                if self.array_index != BACNET_ARRAY_ALL {
                    encode_context_unsigned(writer, 1, self.array_index);
                }
            }

            encode_closing_tag(writer, 1);
        }
    }

    pub fn decode(_reader: &mut Reader) -> Self {
        unimplemented!()
    }
}
