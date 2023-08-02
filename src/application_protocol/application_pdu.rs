use crate::common::{
    error::Error,
    helper::{Reader, Writer},
};

use super::{
    confirmed::{ComplexAck, ConfirmedRequest, SimpleAck},
    unconfirmed::UnconfirmedRequest,
};

// Application Layer Protocol Data Unit
#[derive(Debug)]
pub enum ApplicationPdu<'a> {
    ConfirmedRequest(ConfirmedRequest<'a>),
    UnconfirmedRequest(UnconfirmedRequest),
    ComplexAck(ComplexAck<'a>),
    SimpleAck(SimpleAck),
    // add more here
}

#[derive(Debug)]
#[repr(u8)]
pub enum ApduType {
    ConfirmedServiceRequest = 0,
    UnconfirmedServiceRequest = 1,
    SimpleAck = 2,
    ComplexAck = 3,
    SegmentAck = 4,
    Error = 5,
    Reject = 6,
    Abort = 7,
}

impl From<u8> for ApduType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::ConfirmedServiceRequest,
            1 => Self::UnconfirmedServiceRequest,
            2 => Self::SimpleAck,
            3 => Self::ComplexAck,
            4 => Self::SegmentAck,
            5 => Self::Error,
            6 => Self::Reject,
            7 => Self::Abort,
            _ => panic!("invalid pdu type"),
        }
    }
}

// preshifted by 4 bits
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MaxSegments {
    _0 = 0x00,
    _2 = 0x10,
    _4 = 0x20,
    _8 = 0x30,
    _16 = 0x40,
    _32 = 0x50,
    _64 = 0x60,
    _65 = 0x70, // default
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MaxAdpu {
    _0 = 0x00,
    _128 = 0x01,
    _206 = 0x02,
    _480 = 0x03,
    _1024 = 0x04,
    _1476 = 0x05, // default
}

pub enum PduFlags {
    Server = 0b0001,
    SegmentedResponseAccepted = 0b0010,
    MoreFollows = 0b0100,
    SegmentedMessage = 0b1000,
}

impl<'a> ApplicationPdu<'a> {
    pub fn encode(&self, writer: &mut Writer) {
        match self {
            ApplicationPdu::ConfirmedRequest(req) => req.encode(writer),
            ApplicationPdu::UnconfirmedRequest(req) => req.encode(writer),
            ApplicationPdu::ComplexAck(_) => todo!(),
            ApplicationPdu::SimpleAck(_) => todo!(),
        };
    }

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let byte0 = reader.read_byte(buf);
        let pdu_type: ApduType = (byte0 >> 4).into();
        let pdu_flags = byte0 & 0x0F;
        let segmented_message = (pdu_flags & PduFlags::SegmentedMessage as u8) > 0;
        let _more_follows = (pdu_flags & PduFlags::MoreFollows as u8) > 0;
        let _segmented_response_accepted =
            (pdu_flags & PduFlags::SegmentedResponseAccepted as u8) > 0;

        if segmented_message {
            return Err(Error::SegmentationNotSupported);
        }

        match pdu_type {
            ApduType::ConfirmedServiceRequest => {
                let apdu = ConfirmedRequest::decode(reader, buf);
                Ok(ApplicationPdu::ConfirmedRequest(apdu))
            }
            ApduType::UnconfirmedServiceRequest => {
                let apdu = UnconfirmedRequest::decode(reader, buf);
                Ok(ApplicationPdu::UnconfirmedRequest(apdu))
            }
            ApduType::ComplexAck => {
                let adpu = ComplexAck::decode(reader, buf)?;
                Ok(ApplicationPdu::ComplexAck(adpu))
            }
            ApduType::SimpleAck => {
                let adpu = SimpleAck::decode(reader, buf)?;
                Ok(ApplicationPdu::SimpleAck(adpu))
            }

            _ => panic!("Unsupported pdu type: {:?}", pdu_type),
        }
    }
}
