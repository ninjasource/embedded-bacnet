use crate::application_protocol::confirmed::ConfirmedServiceChoice;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
