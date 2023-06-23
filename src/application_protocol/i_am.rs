use crate::common::{
    error::Error,
    helper::{decode_unsigned, Reader},
    object_id::{ObjectId, ObjectType},
    spec::Segmentation,
    tag::{Tag, TagType},
};

#[derive(Debug)]
pub struct IAm {
    device_id: ObjectId,
    max_apdu: usize,
    segmentation: Segmentation,
    vendor_id: u16,
}

impl IAm {
    pub fn decode(reader: &mut Reader) -> Result<Self, Error> {
        // parse a tag, starting from after the pdu type and service choice, then the object_id
        let tag = Tag::decode(reader);
        if tag.tag_type() != TagType::ObjectId {
            return Err(Error::InvalidValue(
                "expected object_id tag type for IAm device_id field",
            ));
        }
        let device_id = ObjectId::decode(reader, tag.value)?;
        if device_id.object_type != ObjectType::ObjectDevice {
            return Err(Error::InvalidValue(
                "expected device object type for IAm device_id field",
            ));
        }

        // parse a tag then max_apgu
        let tag = Tag::decode(reader);
        if tag.tag_type() != TagType::UnsignedInt {
            return Err(Error::InvalidValue(
                "expected unsigned_int tag type for IAm max_apdu field",
            ));
        }
        let max_apdu = decode_unsigned(reader, tag.value);
        let max_apdu = max_apdu as usize;

        // parse a tag then segmentation
        let tag = Tag::decode(reader);
        if tag.tag_type() != TagType::Enumerated {
            return Err(Error::InvalidValue(
                "expected enumerated tag type for IAm segmentation field",
            ));
        }
        let segmentation = decode_unsigned(reader, tag.value) as u32;
        let segmentation = segmentation.try_into()?;

        // parse a tag then vendor_id
        let tag = Tag::decode(reader);
        if tag.tag_type() != TagType::UnsignedInt {
            return Err(Error::InvalidValue(
                "expected unsigned_int type for IAm vendor_id field",
            ));
        }
        let vendor_id = decode_unsigned(reader, tag.value) as u32;
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
