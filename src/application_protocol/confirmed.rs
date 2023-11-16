use crate::common::{
    error::{Error, Unimplemented},
    helper::decode_unsigned,
    io::{Reader, Writer},
    spec::{ErrorClass, ErrorCode},
    tag::{ApplicationTagNumber, Tag, TagNumber},
};

use super::{
    application_pdu::{ApduType, MaxAdpu, MaxSegments, PduFlags},
    services::{
        change_of_value::SubscribeCov,
        read_property::{ReadProperty, ReadPropertyAck},
        read_property_multiple::{ReadPropertyMultiple, ReadPropertyMultipleAck},
        read_range::{ReadRange, ReadRangeAck},
        write_property::WriteProperty,
    },
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConfirmedRequest<'a> {
    pub max_segments: MaxSegments, // default 65
    pub max_adpu: MaxAdpu,         // default 1476
    pub invoke_id: u8,             // starts at 0
    pub sequence_num: u8,          // default to 0
    pub proposed_window_size: u8,  // default to 0
    pub service: ConfirmedRequestService<'a>,
}

impl<'a> ConfirmedRequest<'a> {
    pub fn new(invoke_id: u8, service: ConfirmedRequestService<'a>) -> Self {
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
        writer.push(self.max_segments.clone() as u8 | self.max_adpu.clone() as u8);
        writer.push(self.invoke_id);

        // NOTE: Segment pdu not supported / implemented

        match &self.service {
            ConfirmedRequestService::ReadProperty(service) => {
                writer.push(ConfirmedServiceChoice::ReadProperty as u8);
                service.encode(writer)
            }
            ConfirmedRequestService::ReadPropertyMultiple(service) => {
                writer.push(ConfirmedServiceChoice::ReadPropMultiple as u8);
                service.encode(writer)
            }
            ConfirmedRequestService::SubscribeCov(service) => {
                writer.push(ConfirmedServiceChoice::SubscribeCov as u8);
                service.encode(writer)
            }
            ConfirmedRequestService::WriteProperty(service) => {
                writer.push(ConfirmedServiceChoice::WriteProperty as u8);
                service.encode(writer)
            }
            ConfirmedRequestService::ReadRange(service) => {
                writer.push(ConfirmedServiceChoice::ReadRange as u8);
                service.encode(writer)
            }
        };
    }

    // the control byte has already been read
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let byte0 = reader.read_byte(buf)?;
        let max_segments: MaxSegments = (byte0 & 0xF0).into();
        let max_adpu: MaxAdpu = (byte0 & 0x0F).into();
        let invoke_id = reader.read_byte(buf)?;

        let choice: ConfirmedServiceChoice = reader.read_byte(buf)?.try_into().map_err(|e| {
            Error::InvalidVariant(("ConfirmedRequest decode ConfirmedServiceChoice", e as u32))
        })?;

        let service = match choice {
            ConfirmedServiceChoice::ReadProperty => {
                let service = ReadProperty::decode(reader, buf)?;
                ConfirmedRequestService::ReadProperty(service)
            }
            ConfirmedServiceChoice::ReadPropMultiple => {
                let service = ReadPropertyMultiple::decode(reader, buf);
                ConfirmedRequestService::ReadPropertyMultiple(service)
            }
            ConfirmedServiceChoice::ReadRange => {
                let service = ReadRange::decode(reader, buf)?;
                ConfirmedRequestService::ReadRange(service)
            }
            ConfirmedServiceChoice::WriteProperty => {
                let service = WriteProperty::decode(reader, buf)?;
                ConfirmedRequestService::WriteProperty(service)
            }
            _ => todo!("Choice not supported: {:?}", choice),
        };

        Ok(Self {
            max_segments,
            max_adpu,
            sequence_num: 0,
            proposed_window_size: 0,
            invoke_id,
            service,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl TryFrom<u8> for ConfirmedServiceChoice {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        match value {
            0 => Ok(Self::AcknowledgeAlarm),
            1 => Ok(Self::CovNotification),
            2 => Ok(Self::EventNotification),
            3 => Ok(Self::GetAlarmSummary),
            4 => Ok(Self::GetEnrollmentSummary),
            5 => Ok(Self::SubscribeCov),
            6 => Ok(Self::AtomicReadFile),
            7 => Ok(Self::AtomicWriteFile),
            8 => Ok(Self::AddListElement),
            9 => Ok(Self::RemoveListElement),
            10 => Ok(Self::CreateObject),
            11 => Ok(Self::DeleteObject),
            12 => Ok(Self::ReadProperty),
            13 => Ok(Self::ReadPropConditional),
            14 => Ok(Self::ReadPropMultiple),
            15 => Ok(Self::WriteProperty),
            16 => Ok(Self::WritePropMultiple),
            17 => Ok(Self::DeviceCommunicationControl),
            18 => Ok(Self::PrivateTransfer),
            19 => Ok(Self::TextMessage),
            20 => Ok(Self::ReinitializeDevice),
            21 => Ok(Self::VtOpen),
            22 => Ok(Self::VtClose),
            23 => Ok(Self::VtData),
            24 => Ok(Self::Authenticate),
            25 => Ok(Self::RequestKey),
            26 => Ok(Self::ReadRange),
            27 => Ok(Self::LifeSafetyOperation),
            28 => Ok(Self::SubscribeCovProperty),
            29 => Ok(Self::GetEventInformation),
            30 => Ok(Self::SubscribeCovPropertyMultiple),
            31 => Ok(Self::CovNotificationMultiple),
            32 => Ok(Self::AuditNotification),
            33 => Ok(Self::AuditLogQuery),
            34 => Ok(Self::MaxBacnetConfirmedService),
            x => Err(x),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SimpleAck {
    pub invoke_id: u8,
    pub service_choice: ConfirmedServiceChoice,
}

impl SimpleAck {
    pub fn encode(&self, writer: &mut Writer) {
        let control = (ApduType::SimpleAck as u8) << 4;
        writer.push(control);
        writer.push(self.invoke_id);
        writer.push(self.service_choice.clone() as u8);
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let invoke_id = reader.read_byte(buf)?;
        let service_choice: ConfirmedServiceChoice =
            reader.read_byte(buf)?.try_into().map_err(|e| {
                Error::InvalidVariant(("SimpleAck decode ConfirmedServiceChoice", e as u32))
            })?;

        Ok(Self {
            invoke_id,
            service_choice,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BacnetError {
    pub invoke_id: u8,
    pub service_choice: ConfirmedServiceChoice,
    pub error_class: ErrorClass,
    pub error_code: ErrorCode,
}

impl BacnetError {
    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let invoke_id = reader.read_byte(buf)?;
        let service_choice: ConfirmedServiceChoice =
            reader.read_byte(buf)?.try_into().map_err(|e| {
                Error::InvalidVariant(("BacnetError decode ConfirmedServiceChoice", e as u32))
            })?;

        let tag = Tag::decode_expected(
            reader,
            buf,
            TagNumber::Application(ApplicationTagNumber::Enumerated),
            "BacnetError error class",
        )?;
        let value = decode_unsigned(tag.value, reader, buf)? as u32;
        let error_class =
            ErrorClass::try_from(value).map_err(|e| Error::InvalidVariant(("ErrorClass", e)))?;

        let tag = Tag::decode_expected(
            reader,
            buf,
            TagNumber::Application(ApplicationTagNumber::Enumerated),
            "BacnetError error code",
        )?;
        let value = decode_unsigned(tag.value, reader, buf)? as u32;
        let error_code =
            ErrorCode::try_from(value).map_err(|e| Error::InvalidVariant(("ErrorCode", e)))?;

        Ok(Self {
            invoke_id,
            service_choice,
            error_class,
            error_code,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ComplexAck<'a> {
    pub invoke_id: u8,
    pub service: ComplexAckService<'a>,
}

impl<'a> ComplexAck<'a> {
    pub fn encode(&self, writer: &mut Writer) {
        let control = (ApduType::ComplexAck as u8) << 4;
        writer.push(control);
        writer.push(self.invoke_id);

        match &self.service {
            ComplexAckService::ReadProperty(service) => service.encode(writer),
            ComplexAckService::ReadPropertyMultiple(service) => service.encode(writer),
            ComplexAckService::ReadRange(service) => service.encode(writer),
        }
    }

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let invoke_id = reader.read_byte(buf)?;
        let choice: ConfirmedServiceChoice = reader.read_byte(buf)?.try_into().map_err(|e| {
            Error::InvalidVariant(("ComplexAck decode ConfirmedServiceChoice", e as u32))
        })?;

        let service = match choice {
            ConfirmedServiceChoice::ReadProperty => {
                let apdu = ReadPropertyAck::decode(reader, buf)?;
                ComplexAckService::ReadProperty(apdu)
            }
            ConfirmedServiceChoice::ReadPropMultiple => {
                let buf = &buf[reader.index..reader.end];
                ComplexAckService::ReadPropertyMultiple(ReadPropertyMultipleAck::new_from_buf(buf))
            }
            ConfirmedServiceChoice::ReadRange => {
                let apdu = ReadRangeAck::decode(reader, buf)?;
                ComplexAckService::ReadRange(apdu)
            }
            s => {
                return Err(Error::Unimplemented(Unimplemented::ConfirmedServiceChoice(
                    s,
                )))
            }
        };

        Ok(Self { invoke_id, service })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ComplexAckService<'a> {
    ReadProperty(ReadPropertyAck<'a>),
    ReadPropertyMultiple(ReadPropertyMultipleAck<'a>),
    ReadRange(ReadRangeAck<'a>),
    // add more here
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConfirmedRequestService<'a> {
    ReadProperty(ReadProperty),
    ReadPropertyMultiple(ReadPropertyMultiple<'a>),
    SubscribeCov(SubscribeCov),
    WriteProperty(WriteProperty<'a>),
    ReadRange(ReadRange),
    // add more here (see ConfirmedServiceChoice enum)
}
