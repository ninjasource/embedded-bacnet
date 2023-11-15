use crate::application_protocol::confirmed::ConfirmedServiceChoice;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Error {
    Length(&'static str),
    InvalidValue(&'static str),
    Unknown,
    UnimplementedConfirmedServiceChoice(ConfirmedServiceChoice),
    SegmentationNotSupported,
    UnexpectedInvokeId,
    Io,
    ApduTypeNotSupported,
}
