use crate::{
    application_protocol::{
        confirmed::{ComplexAck, ConfirmedBacnetError, ConfirmedRequest, SegmentAck, SimpleAck},
        segment::Segment,
        unconfirmed::UnconfirmedRequest,
    },
    common::{
        error::{self, Error},
        io::{Reader, Writer},
    },
};

// Application Layer Protocol Data Unit
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ApplicationPdu<'a> {
    ConfirmedRequest(ConfirmedRequest<'a>),
    UnconfirmedRequest(UnconfirmedRequest<'a>),
    ComplexAck(ComplexAck<'a>),
    SimpleAck(SimpleAck),
    Error(ConfirmedBacnetError),
    Segment(Segment<'a>),
    SegmentAck(SegmentAck),
    // add more here (see ApduType)
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl TryFrom<u8> for ApduType {
    type Error = error::Error;

    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0 => Ok(Self::ConfirmedServiceRequest),
            1 => Ok(Self::UnconfirmedServiceRequest),
            2 => Ok(Self::SimpleAck),
            3 => Ok(Self::ComplexAck),
            4 => Ok(Self::SegmentAck),
            5 => Ok(Self::Error),
            6 => Ok(Self::Reject),
            7 => Ok(Self::Abort),
            x => Err(Error::InvalidVariant(("ApduType", x as u32))),
        }
    }
}

// preshifted by 4 bits
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

impl From<u8> for MaxSegments {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::_0,
            0x10 => Self::_2,
            0x20 => Self::_4,
            0x30 => Self::_8,
            0x40 => Self::_16,
            0x50 => Self::_32,
            0x60 => Self::_64,
            0x70 => Self::_65,
            _ => Self::_65,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum MaxAdpu {
    _0 = 0x00,
    _128 = 0x01,
    _206 = 0x02,
    _480 = 0x03,
    _1024 = 0x04,
    _1476 = 0x05, // default
}

impl From<u8> for MaxAdpu {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::_0,
            0x01 => Self::_128,
            0x02 => Self::_206,
            0x03 => Self::_480,
            0x04 => Self::_1024,
            0x05 => Self::_1476,
            _ => Self::_1476,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PduFlags {
    Server = 0b0001,
    SegmentedResponseAccepted = 0b0010,
    MoreFollows = 0b0100,
    SegmentedMessage = 0b1000,
}

impl<'a> ApplicationPdu<'a> {
    pub fn encode(&self, writer: &mut Writer) {
        match self {
            Self::ConfirmedRequest(req) => req.encode(writer),
            Self::UnconfirmedRequest(req) => req.encode(writer),
            Self::ComplexAck(req) => req.encode(writer),
            Self::SimpleAck(ack) => ack.encode(writer),
            Self::SegmentAck(ack) => ack.encode(writer),
            Self::Segment(segment) => segment.encode(writer),
            Self::Error(_) => todo!(),
        };
    }

    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let byte0 = reader.read_byte(buf)?;
        let pdu_type: ApduType = (byte0 >> 4).try_into()?;
        let pdu_flags = byte0 & 0x0F;
        let segmented_message = (pdu_flags & PduFlags::SegmentedMessage as u8) > 0;
        let more_follows = (pdu_flags & PduFlags::MoreFollows as u8) > 0;
        let _segmented_response_accepted =
            (pdu_flags & PduFlags::SegmentedResponseAccepted as u8) > 0;

        if segmented_message {
            let segment = Segment::decode(more_follows, pdu_type, reader, buf)?;
            return Ok(Self::Segment(segment));
        }

        match pdu_type {
            ApduType::ConfirmedServiceRequest => {
                let apdu = ConfirmedRequest::decode(reader, buf)?;
                Ok(Self::ConfirmedRequest(apdu))
            }
            ApduType::UnconfirmedServiceRequest => {
                let apdu = UnconfirmedRequest::decode(reader, buf)?;
                Ok(Self::UnconfirmedRequest(apdu))
            }
            ApduType::ComplexAck => {
                let adpu = ComplexAck::decode(reader, buf)?;
                Ok(Self::ComplexAck(adpu))
            }
            ApduType::SimpleAck => {
                let adpu = SimpleAck::decode(reader, buf)?;
                Ok(Self::SimpleAck(adpu))
            }
            ApduType::SegmentAck => {
                let adpu = SegmentAck::decode(reader, buf)?;
                Ok(Self::SegmentAck(adpu))
            }
            ApduType::Error => {
                let apdu = ConfirmedBacnetError::decode(reader, buf)?;
                Ok(Self::Error(apdu))
            }
            apdu_type => Err(Error::ApduTypeNotSupported(apdu_type)),
        }
    }
}
