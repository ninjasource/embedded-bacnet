use crate::application_protocol::confirmed::ConfirmedServiceChoice;

#[derive(Debug)]
pub enum Error {
    Length(&'static str),
    InvalidValue(&'static str),
    Unknown,
    UnimplementedConfirmedServiceChoice(ConfirmedServiceChoice),
    SegmentationNotSupported,
    UnexpectedInvokeId,
    Io,
}
