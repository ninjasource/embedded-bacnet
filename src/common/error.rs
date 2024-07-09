use crate::{
    application_protocol::{
        application_pdu::ApduType, confirmed::ConfirmedServiceChoice,
        services::read_range::ReadRangeValueType, unconfirmed::UnconfirmedServiceChoice,
    },
    common::tag::{ApplicationTagNumber, Tag, TagNumber},
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    Length((&'static str, u32)),
    InvalidValue(&'static str),
    InvalidVariant((&'static str, u32)),
    Unimplemented(Unimplemented),
    SegmentationNotSupported,
    ApduTypeNotSupported(ApduType),
    ExpectedTag(ExpectedTag),
    ExpectedOpeningTag(TagNumber),
    TagNotSupported((&'static str, TagNumber)),
    TagValueInvalid((&'static str, Tag, u32)),
    ReaderEof(usize),
    ConvertDataLink(&'static str),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Unimplemented {
    ConfirmedServiceChoice(ConfirmedServiceChoice),
    UnconfirmedServiceChoice(UnconfirmedServiceChoice),
    ReadRangeValueType(ReadRangeValueType),
    ApplicationTagNumber(ApplicationTagNumber),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpectedTag {
    pub context: &'static str,
    pub expected: TagNumber,
    pub actual: TagNumber,
}
