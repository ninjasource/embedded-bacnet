use crate::common::io::{Reader, Writer};

use super::{
    application_pdu::ApduType,
    services::{change_of_value::CovNotification, i_am::IAm, who_is::WhoIs},
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UnconfirmedRequest<'a> {
    WhoIs(WhoIs),
    IAm(IAm),
    CovNotification(CovNotification<'a>),
}

impl<'a> UnconfirmedRequest<'a> {
    pub fn encode(&self, writer: &mut Writer) {
        writer.push((ApduType::UnconfirmedServiceRequest as u8) << 4);

        match &self {
            Self::IAm(_) => todo!(),
            Self::WhoIs(payload) => payload.encode(writer),
            Self::CovNotification(_) => todo!(),
        }
    }

    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Self {
        let choice: UnconfirmedServiceChoice = reader.read_byte(buf).into();
        match choice {
            UnconfirmedServiceChoice::IAm => {
                let apdu = IAm::decode(reader, buf).unwrap();
                UnconfirmedRequest::IAm(apdu)
            }
            UnconfirmedServiceChoice::WhoIs => {
                let apdu = WhoIs::decode(reader, buf);
                UnconfirmedRequest::WhoIs(apdu)
            }
            UnconfirmedServiceChoice::CovNotification => {
                let apdu = CovNotification::decode(reader, buf).unwrap();
                UnconfirmedRequest::CovNotification(apdu)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
