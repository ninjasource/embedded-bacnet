use crate::common::helper::{Buffer, Reader};

use super::{i_am::IAm, who_is::WhoIs};

// Application Layer Protocol Data Unit
#[derive(Debug)]
pub enum ApplicationPdu {
    ConfirmedRequest(ConfirmedRequest),
    UnconfirmedRequest(UnconfirmedRequest),
    // add more here
}

#[repr(u8)]
pub enum PduType {
    ConfirmedServiceRequest = 0,
    UnconfirmedServiceRequest = 1,
    SimpleAck = 2,
    ComplexAck = 3,
    SegmentAck = 4,
    Error = 5,
    Reject = 6,
    Abort = 7,
}

impl From<u8> for PduType {
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
    _65 = 0x70,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MaxAdpu {
    _0 = 0x00,
    _128 = 0x01,
    _206 = 0x02,
    _480 = 0x03,
    _1024 = 0x04,
    _1476 = 0x05,
}

#[derive(Debug)]
#[repr(u8)]
pub enum ConfirmedServiceChoice {
    // alarm and event services
    AcknowledgeAlarm = 0,
    AuditNotification = 32,
    CovNotification = 1,
    CovNotificationMultiple = 31,
    EventNotification = 2,
    GetAlarmSummary = 3,
    GetEnrollmentSummary = 4,
    GetEventInformation = 29,
    LifeSafetyOperation = 27,
    SubscribeCov = 5,
    SubscribeCovProperty = 28,
    SubscribeCovPropertyMultiple = 30,

    // file access services
    AtomicReadFile = 6,
    AtomicWriteFile = 7,

    // object access services
    AddListElement = 8,
    RemoveListElement = 9,
    CreateObject = 10,
    DeleteObject = 11,
    ReadProperty = 12,
    ReadPropConditional = 13,
    ReadPropMultiple = 14,
    ReadRange = 26,
    WriteProperty = 15,
    WritePropMultiple = 16,
    AuditLogQuery = 33,

    // remote device management services
    DeviceCommunicationControl = 17,
    PrivateTransfer = 18,
    TextMessage = 19,
    ReinitializeDevice = 20,

    // virtual terminal services
    VtOpen = 21,
    VtClose = 22,
    VtData = 23,

    // security services
    Authenticate = 24,
    RequestKey = 25,

    // services added after 1995
    // readRange [26] see Object Access Services
    // lifeSafetyOperation [27] see Alarm and Event Services
    // subscribeCOVProperty [28] see Alarm and Event Services
    // getEventInformation [29] see Alarm and Event Services

    // services added after 2012
    // subscribe-cov-property-multiple [30] see Alarm and Event Services
    // confirmed-cov-notification-multiple [31] see Alarm and Event Services

    // services added after 2016
    // confirmed-audit-notification [32] see Alarm and Event Services
    // audit-log-query [33] see Object Access Services
    MaxBacnetConfirmedService = 34,
}

impl ApplicationPdu {
    pub fn encode(&self, buffer: &mut Buffer) {
        match self {
            ApplicationPdu::ConfirmedRequest(req) => req.encode(buffer),
            ApplicationPdu::UnconfirmedRequest(req) => req.encode(buffer),
        };
    }

    pub fn decode(reader: &mut Reader) -> Self {
        let byte0 = reader.read_byte();
        let pdu_type: PduType = (byte0 >> 4).into();

        match pdu_type {
            PduType::ConfirmedServiceRequest => {
                let apdu = ConfirmedRequest::decode(reader);
                ApplicationPdu::ConfirmedRequest(apdu)
            }
            PduType::UnconfirmedServiceRequest => {
                let apdu = UnconfirmedRequest::decode(reader);
                ApplicationPdu::UnconfirmedRequest(apdu)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct ConfirmedRequest {
    service_choice: ConfirmedServiceChoice,
    invoke_id: u8,
}

impl ConfirmedRequest {
    pub fn encode(&self, buffer: &mut Buffer) {
        buffer.push((PduType::ConfirmedServiceRequest as u8) << 4);
    }

    pub fn decode(_reader: &mut Reader) -> Self {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum UnconfirmedRequest {
    WhoIs(WhoIs),
    IAm(IAm),
}

impl UnconfirmedRequest {
    pub fn encode(&self, buffer: &mut Buffer) {
        buffer.push((PduType::UnconfirmedServiceRequest as u8) << 4);

        match &self {
            Self::IAm(_) => todo!(),
            Self::WhoIs(payload) => payload.encode(buffer),
        }
    }

    pub fn decode(reader: &mut Reader) -> Self {
        let choice: UnconfirmedServiceChoice = reader.read_byte().into();
        match choice {
            UnconfirmedServiceChoice::IAm => {
                let apdu = IAm::decode(reader).unwrap();
                UnconfirmedRequest::IAm(apdu)
            }
            UnconfirmedServiceChoice::WhoIs => {
                let apdu = WhoIs::decode(reader);
                UnconfirmedRequest::WhoIs(apdu)
            }
            _ => unimplemented!(),
        }
    }
}

pub enum UnconfirmedServiceChoice {
    IAm = 0,
    IHave = 1,
    CovNotification = 2,
    EventNotification = 3,
    PrivateTransfer = 4,
    TextMessage = 5,
    TimeSynchronization = 6,
    WhoHas = 7,
    WhoIs = 8,
    UtcTimeSynchronization = 9,

    // addendum 2010-aa
    WriteGroup = 10,

    // addendum 2012-aq
    CovNotificationMultiple = 11,

    // addendum 2016-bi
    AuditNotification = 12,

    // addendum 2016-bz
    WhoAmI = 13,
    YouAre = 14,

    // Other services to be added as they are defined.
    // All choice values in this production are reserved
    // for definition by ASHRAE.
    // Proprietary extensions are made by using the
    // UnconfirmedPrivateTransfer service. See Clause 23.
    MaxBacnetUnconfirmedService = 15,
}

impl From<u8> for UnconfirmedServiceChoice {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::IAm,
            1 => Self::IHave,
            2 => Self::CovNotification,
            3 => Self::EventNotification,
            4 => Self::PrivateTransfer,
            5 => Self::TextMessage,
            6 => Self::TimeSynchronization,
            7 => Self::WhoHas,
            8 => Self::WhoIs,
            9 => Self::UtcTimeSynchronization,
            10 => Self::WriteGroup,
            11 => Self::CovNotificationMultiple,
            12 => Self::AuditNotification,
            13 => Self::WhoAmI,
            14 => Self::YouAre,
            15 => Self::MaxBacnetUnconfirmedService,
            _ => panic!("invalid unconfirmed service choice"),
        }
    }
}
