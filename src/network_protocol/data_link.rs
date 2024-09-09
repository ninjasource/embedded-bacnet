use crate::{
    application_protocol::{application_pdu::ApplicationPdu, confirmed::ConfirmedRequest},
    common::{
        error::Error,
        io::{Reader, Writer},
    },
    network_protocol::network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
};

// Bacnet Virtual Link Control
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DataLink<'a> {
    pub function: DataLinkFunction,
    pub npdu: Option<NetworkPdu<'a>>,
    pub raw_payload: &'a [u8],
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum DataLinkFunction {
    Result = 0,
    WriteBroadcastDistributionTable = 1,
    ReadBroadcastDistTable = 2,
    ReadBroadcastDistTableAck = 3,
    ForwardedNpdu = 4,
    RegisterForeignDevice = 5,
    ReadForeignDeviceTable = 6,
    ReadForeignDeviceTableAck = 7,
    DeleteForeignDeviceTableEntry = 8,
    DistributeBroadcastToNetwork = 9,
    OriginalUnicastNpdu = 10,
    OriginalBroadcastNpdu = 11,
}

impl TryFrom<u8> for DataLinkFunction {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Result),
            1 => Ok(Self::WriteBroadcastDistributionTable),
            2 => Ok(Self::ReadBroadcastDistTable),
            3 => Ok(Self::ReadBroadcastDistTableAck),
            4 => Ok(Self::ForwardedNpdu),
            5 => Ok(Self::RegisterForeignDevice),
            6 => Ok(Self::ReadForeignDeviceTable),
            7 => Ok(Self::ReadForeignDeviceTableAck),
            8 => Ok(Self::DeleteForeignDeviceTableEntry),
            9 => Ok(Self::DistributeBroadcastToNetwork),
            10 => Ok(Self::OriginalUnicastNpdu),
            11 => Ok(Self::OriginalBroadcastNpdu),
            x => Err(x),
        }
    }
}

const BVLL_TYPE_BACNET_IP: u8 = 0x81;

impl<'a> DataLink<'a> {
    //    const BVLC_ORIGINAL_UNICAST_NPDU: u8 = 10;
    //    const BVLC_ORIGINAL_BROADCAST_NPDU: u8 = 11;

    pub fn new(function: DataLinkFunction, npdu: Option<NetworkPdu<'a>>) -> Self {
        Self {
            function,
            npdu,
            raw_payload: &[],
        }
    }

    pub fn new_confirmed_req(req: ConfirmedRequest<'a>) -> Self {
        let apdu = ApplicationPdu::ConfirmedRequest(req);
        let message = NetworkMessage::Apdu(apdu);
        let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
        DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu))
    }

    pub fn encode(&self, writer: &mut Writer) {
        writer.push(BVLL_TYPE_BACNET_IP);
        writer.push(self.function.clone() as u8);
        match &self.function {
            DataLinkFunction::OriginalBroadcastNpdu | DataLinkFunction::OriginalUnicastNpdu => {
                writer.extend_from_slice(&[0, 0]); // length placeholder
                self.npdu.as_ref().unwrap().encode(writer); // should be ok to unwrap here since it has already been checked
                Self::update_len(writer);
            }
            _ => todo!(),
        }
    }

    fn update_len(writer: &mut Writer) {
        let len = writer.index as u16;
        let src = len.to_be_bytes();
        writer.buf[2..4].copy_from_slice(&src);
    }

    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let bvll_type = reader.read_byte(buf)?;
        if bvll_type != BVLL_TYPE_BACNET_IP {
            return Err(Error::InvalidValue("only BACNET_IP supported"));
        }

        let function = reader
            .read_byte(buf)?
            .try_into()
            .map_err(|_| Error::InvalidValue("bvll function value out of range"))?;
        let len: u16 = u16::from_be_bytes(reader.read_bytes(buf)?);

        if len as usize > buf.len() {
            return Err(Error::Length((
                "read buffer too small to fit entire bacnet payload",
                len as u32,
            )));
        }
        reader.set_len(len as usize);

        let npdu_start_index = reader.index;
        let npdu = match function {
            // see h_bbmd.c for all the types (only 2 are supported here)
            DataLinkFunction::OriginalBroadcastNpdu | DataLinkFunction::OriginalUnicastNpdu => {
                Some(NetworkPdu::decode(reader, buf)?)
            }
            _ => None,
        };

        Ok(Self {
            function,
            npdu,
            raw_payload: &buf[npdu_start_index..],
        })
    }
}
