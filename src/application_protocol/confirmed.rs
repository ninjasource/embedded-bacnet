use crate::common::{
    error::Error,
    helper::{Reader, Writer},
};

use super::{
    application_pdu::{ApduType, MaxAdpu, MaxSegments, PduFlags},
    services::{
        change_of_value::SubscribeCov,
        read_property::{ReadProperty, ReadPropertyAck},
        read_property_multiple::{ReadPropertyMultiple, ReadPropertyMultipleAck},
        write_property::WriteProperty,
    },
};

#[derive(Debug)]
pub struct ConfirmedRequest<'a> {
    pub max_segments: MaxSegments, // default 65
    pub max_adpu: MaxAdpu,         // default 1476
    pub invoke_id: u8,             // starts at 0
    pub sequence_num: u8,          // default to 0
    pub proposed_window_size: u8,  // default to 0
    pub service: ConfirmedRequestSerivice<'a>,
}

impl<'a> ConfirmedRequest<'a> {
    pub fn new(invoke_id: u8, service: ConfirmedRequestSerivice<'a>) -> Self {
        Self {
            max_segments: MaxSegments::_65,
            max_adpu: MaxAdpu::_1476,
            invoke_id,
            sequence_num: 0,
            proposed_window_size: 0,
            service,
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        let max_segments_flag = match self.max_segments {
            MaxSegments::_0 => 0,
            _ => PduFlags::SegmentedResponseAccepted as u8,
        };

        let control = ((ApduType::ConfirmedServiceRequest as u8) << 4) | max_segments_flag;
        writer.push(control);
        writer.push(self.max_segments as u8 | self.max_adpu as u8);
        writer.push(self.invoke_id);

        // NOTE: Segment pdu not supported / implemented

        match &self.service {
            ConfirmedRequestSerivice::ReadProperty(service) => {
                writer.push(ConfirmedServiceChoice::ReadProperty as u8);
                service.encode(writer)
            }
            ConfirmedRequestSerivice::ReadPropertyMultiple(service) => {
                writer.push(ConfirmedServiceChoice::ReadPropMultiple as u8);
                service.encode(writer)
            }
            ConfirmedRequestSerivice::SubscribeCov(service) => {
                writer.push(ConfirmedServiceChoice::SubscribeCov as u8);
                service.encode(writer)
            }
            ConfirmedRequestSerivice::WriteProperty(service) => {
                writer.push(ConfirmedServiceChoice::WriteProperty as u8);
                service.encode(writer)
            }
        };
    }

    pub fn decode(_reader: &mut Reader, _buf: &[u8]) -> Self {
        unimplemented!()
    }
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

impl From<u8> for ConfirmedServiceChoice {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::AcknowledgeAlarm,
            1 => Self::CovNotification,
            2 => Self::EventNotification,
            3 => Self::GetAlarmSummary,
            4 => Self::GetEnrollmentSummary,
            5 => Self::SubscribeCov,
            6 => Self::AtomicReadFile,
            7 => Self::AtomicWriteFile,
            8 => Self::AddListElement,
            9 => Self::RemoveListElement,
            10 => Self::CreateObject,
            11 => Self::DeleteObject,
            12 => Self::ReadProperty,
            13 => Self::ReadPropConditional,
            14 => Self::ReadPropMultiple,
            15 => Self::WriteProperty,
            16 => Self::WritePropMultiple,
            17 => Self::DeviceCommunicationControl,
            18 => Self::PrivateTransfer,
            19 => Self::TextMessage,
            20 => Self::ReinitializeDevice,
            21 => Self::VtOpen,
            22 => Self::VtClose,
            23 => Self::VtData,
            24 => Self::Authenticate,
            25 => Self::RequestKey,
            26 => Self::ReadRange,
            27 => Self::LifeSafetyOperation,
            28 => Self::SubscribeCovProperty,
            29 => Self::GetEventInformation,
            30 => Self::SubscribeCovPropertyMultiple,
            31 => Self::CovNotificationMultiple,
            32 => Self::AuditNotification,
            33 => Self::AuditLogQuery,
            34 => Self::MaxBacnetConfirmedService,
            _ => panic!("invalid confirmed service choice"),
        }
    }
}

#[derive(Debug)]
pub struct SimpleAck {
    pub invoke_id: u8,
    pub service_choice: ConfirmedServiceChoice,
}

impl SimpleAck {
    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let invoke_id = reader.read_byte(buf);
        let service_choice = reader.read_byte(buf).into();

        Ok(Self {
            invoke_id,
            service_choice,
        })
    }
}

#[derive(Debug)]
pub struct ComplexAck<'a> {
    pub invoke_id: u8,
    pub service: ComplexAckService<'a>,
}

impl<'a> ComplexAck<'a> {
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let invoke_id = reader.read_byte(buf);
        let choice = reader.read_byte(buf).into();

        let service = match choice {
            ConfirmedServiceChoice::ReadProperty => {
                let apdu = ReadPropertyAck::decode(reader, buf);
                ComplexAckService::ReadProperty(apdu)
            }
            ConfirmedServiceChoice::ReadPropMultiple => {
                ComplexAckService::ReadPropertyMultiple(ReadPropertyMultipleAck {})
            }
            s => return Err(Error::UnimplementedConfirmedServiceChoice(s)),
        };

        Ok(Self { invoke_id, service })
    }
}

#[derive(Debug)]
pub enum ComplexAckService<'a> {
    ReadProperty(ReadPropertyAck<'a>),
    ReadPropertyMultiple(ReadPropertyMultipleAck),
    // add more here
}

#[derive(Debug)]
pub enum ConfirmedRequestSerivice<'a> {
    ReadProperty(ReadProperty),
    ReadPropertyMultiple(ReadPropertyMultiple<'a>),
    SubscribeCov(SubscribeCov),
    WriteProperty(WriteProperty<'a>),
    // add more here (see ConfirmedServiceChoice enum)
}
