use crate::{
    application_protocol::{application_pdu::MaxAdpu, unconfirmed::UnconfirmedServiceChoice},
    common::{
        error::Error,
        helper::{
            decode_unsigned, encode_application_enumerated, encode_application_object_id,
            encode_application_unsigned,
        },
        io::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
        spec::Segmentation,
        tag::{ApplicationTagNumber, Tag, TagNumber},
    },
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IAm {
    pub device_id: ObjectId,
    pub max_apdu: MaxAdpu,
    pub segmentation: Segmentation,
    pub vendor_id: u16,
}

impl IAm {
    pub fn encode(&self, writer: &mut Writer) {
        writer.push(UnconfirmedServiceChoice::IAm as u8);
        encode_application_object_id(writer, &self.device_id);
        encode_application_unsigned(writer, self.max_apdu.clone() as u64);
        encode_application_enumerated(writer, self.segmentation.clone() as u32);
        encode_application_unsigned(writer, self.vendor_id as u64);
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        // parse a tag, starting from after the pdu type and service choice, then the object_id
        let tag = Tag::decode(reader, buf)?;
        if tag.number != TagNumber::Application(ApplicationTagNumber::ObjectId) {
            return Err(Error::InvalidValue(
                "expected object_id tag type for IAm device_id field",
            ));
        }
        let device_id = ObjectId::decode(tag.value, reader, buf)?;
        if device_id.object_type != ObjectType::ObjectDevice {
            return Err(Error::InvalidValue(
                "expected device object type for IAm device_id field",
            ));
        }

        // parse a tag then max_apgu
        let tag = Tag::decode(reader, buf)?;
        if tag.number != TagNumber::Application(ApplicationTagNumber::UnsignedInt) {
            return Err(Error::InvalidValue(
                "expected unsigned_int tag type for IAm max_apdu field",
            ));
        }
        let max_apdu = decode_unsigned(tag.value, reader, buf)?;
        let max_apdu: MaxAdpu = (max_apdu as u8).into();

        // parse a tag then segmentation
        let tag = Tag::decode(reader, buf)?;
        if tag.number != TagNumber::Application(ApplicationTagNumber::Enumerated) {
            return Err(Error::InvalidValue(
                "expected enumerated tag type for IAm segmentation field",
            ));
        }
        let segmentation = decode_unsigned(tag.value, reader, buf)? as u32;
        let segmentation = segmentation.try_into()?;

        // parse a tag then vendor_id
        let tag = Tag::decode(reader, buf)?;
        if tag.number != TagNumber::Application(ApplicationTagNumber::UnsignedInt) {
            return Err(Error::InvalidValue(
                "expected unsigned_int type for IAm vendor_id field",
            ));
        }
        let vendor_id = decode_unsigned(tag.value, reader, buf)? as u32;
        if vendor_id > u16::MAX as u32 {
            return Err(Error::InvalidValue("vendor_id out of range for IAm"));
        }
        let vendor_id = vendor_id as u16;

        Ok(Self {
            device_id,
            max_apdu,
            segmentation,
            vendor_id,
        })
    }
}
