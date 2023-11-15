use crate::application_protocol::{
    application_pdu::ApduType, confirmed::ConfirmedServiceChoice,
    unconfirmed::UnconfirmedServiceChoice,
};

use super::tag::{Tag, TagNumber};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    Length(&'static str),
    InvalidValue(&'static str),
    InvalidVariant((&'static str, u32)),
    Unknown,
    Unimplemented(Unimplemented),
    SegmentationNotSupported,
    UnexpectedInvokeId,
    Io,
    ApduTypeNotSupported(ApduType),
    ExpectedTag(ExpectedTag),
    TagNotSupported((&'static str, TagNumber)),
    TagValueInvalid((&'static str, Tag, u32)),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Unimplemented {
    ConfirmedServiceChoice(ConfirmedServiceChoice),
    UnconfirmedServiceChoice(UnconfirmedServiceChoice),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpectedTag {
    pub context: &'static str,
    pub expected: TagNumber,
    pub actual: TagNumber,
}
