use crate::{
    application_protocol::{
        application_pdu::ApduType,
        services::{
            change_of_value::CovNotification, i_am::IAm, time_synchronization::TimeSynchronization,
            who_is::WhoIs,
        },
    },
    common::{
        error::{Error, Unimplemented},
        io::{Reader, Writer},
    },
};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UnconfirmedRequest<'a> {
    WhoIs(WhoIs),
    IAm(IAm),
    CovNotification(CovNotification<'a>),
    TimeSynchronization(TimeSynchronization),
}

impl<'a> UnconfirmedRequest<'a> {
    pub fn encode(&self, writer: &mut Writer) {
        writer.push((ApduType::UnconfirmedServiceRequest as u8) << 4);

        match &self {
            Self::IAm(payload) => payload.encode(writer),
            Self::WhoIs(payload) => payload.encode(writer),
            Self::CovNotification(_) => todo!(),
            Self::TimeSynchronization(payload) => payload.encode(writer),
        }
    }

    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let choice: UnconfirmedServiceChoice = reader
            .read_byte(buf)?
            .try_into()
            .map_err(|x| Error::InvalidVariant(("UnconfirmedRequest choice", x as u32)))?;
        match choice {
            UnconfirmedServiceChoice::IAm => {
                let apdu = IAm::decode(reader, buf)?;
                Ok(Self::IAm(apdu))
            }
            UnconfirmedServiceChoice::WhoIs => {
                let apdu = WhoIs::decode(reader, buf);
                Ok(Self::WhoIs(apdu))
            }
            UnconfirmedServiceChoice::CovNotification => {
                let apdu = CovNotification::decode(reader, buf)?;
                Ok(Self::CovNotification(apdu))
            }
            x => Err(Error::Unimplemented(
                Unimplemented::UnconfirmedServiceChoice(x),
            )),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl TryFrom<u8> for UnconfirmedServiceChoice {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, u8> {
        match value {
            0 => Ok(Self::IAm),
            1 => Ok(Self::IHave),
            2 => Ok(Self::CovNotification),
            3 => Ok(Self::EventNotification),
            4 => Ok(Self::PrivateTransfer),
            5 => Ok(Self::TextMessage),
            6 => Ok(Self::TimeSynchronization),
            7 => Ok(Self::WhoHas),
            8 => Ok(Self::WhoIs),
            9 => Ok(Self::UtcTimeSynchronization),
            10 => Ok(Self::WriteGroup),
            11 => Ok(Self::CovNotificationMultiple),
            12 => Ok(Self::AuditNotification),
            13 => Ok(Self::WhoAmI),
            14 => Ok(Self::YouAre),
            15 => Ok(Self::MaxBacnetUnconfirmedService),
            x => Err(x),
        }
    }
}
