use crate::{
    application_protocol::{
        application_pdu::{ApplicationPdu, ComplexAckService, ConfirmedRequest},
        read_property::ReadPropertyAck,
        read_property_multiple::ReadPropertyMultipleAck,
    },
    common::{
        error::Error,
        helper::{Reader, Writer},
    },
};

use super::network_pdu::{MessagePriority, NetworkMessage, NetworkPdu};

// Bacnet Virtual Link Control
#[derive(Debug)]
pub struct DataLink<'a> {
    pub function: DataLinkFunction<'a>,
}

#[derive(Debug)]
pub enum DataLinkFunction<'a> {
    OriginalBroadcastNpdu(NetworkPdu<'a>),
    OriginalUnicastNpdu(NetworkPdu<'a>),
}

impl<'a> DataLink<'a> {
    const BVLL_TYPE_BACNET_IP: u8 = 0x81;
    const BVLC_ORIGINAL_UNICAST_NPDU: u8 = 10;
    const BVLC_ORIGINAL_BROADCAST_NPDU: u8 = 11;

    pub fn new(function: DataLinkFunction<'a>) -> Self {
        Self { function }
    }

    pub fn new_confirmed_req(req: ConfirmedRequest<'a>) -> Self {
        let apdu = ApplicationPdu::ConfirmedRequest(req);
        let message = NetworkMessage::Apdu(apdu);
        let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
        DataLink::new(DataLinkFunction::OriginalUnicastNpdu(npdu))
    }

    fn get_ack(&self) -> Option<&ComplexAckService> {
        match &self.function {
            DataLinkFunction::OriginalUnicastNpdu(x) => match &x.network_message {
                NetworkMessage::Apdu(apdu) => match &apdu {
                    ApplicationPdu::ComplexAck(ack) => Some(&ack.service),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    pub fn get_read_property_ack(&self) -> Option<&ReadPropertyAck> {
        match self.get_ack() {
            Some(ack) => match ack {
                ComplexAckService::ReadProperty(ack) => Some(ack),
                _ => None,
            },
            None => None,
        }
    }

    pub fn get_read_property_multiple_ack(&self) -> Option<&ReadPropertyMultipleAck> {
        match self.get_ack() {
            Some(ack) => match ack {
                ComplexAckService::ReadPropertyMultiple(ack) => Some(ack),
                _ => None,
            },
            None => None,
        }
    }

    pub fn encode(&self, buffer: &mut Writer) {
        buffer.push(Self::BVLL_TYPE_BACNET_IP);
        match &self.function {
            DataLinkFunction::OriginalBroadcastNpdu(npdu) => {
                buffer.push(Self::BVLC_ORIGINAL_BROADCAST_NPDU);
                buffer.extend_from_slice(&[0, 0]); // length placeholder
                npdu.encode(buffer);
                Self::update_len(buffer);
            }
            DataLinkFunction::OriginalUnicastNpdu(npdu) => {
                buffer.push(Self::BVLC_ORIGINAL_UNICAST_NPDU);
                buffer.extend_from_slice(&[0, 0]); // length placeholder
                npdu.encode(buffer);
                Self::update_len(buffer);
            }
        }
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let bvll_type = reader.read_byte(buf);
        if bvll_type != Self::BVLL_TYPE_BACNET_IP {
            panic!("only BACNET_IP supported");
        }

        let npdu_type = reader.read_byte(buf);
        let len: u16 = u16::from_be_bytes(reader.read_bytes(buf));

        if len as usize > buf.len() {
            return Err(Error::Length(
                "read buffer too small to fit entire bacnet payload",
            ));
        }
        reader.set_len(len as usize);

        let npdu = NetworkPdu::decode(reader, buf)?;

        let data_link = match npdu_type {
            Self::BVLC_ORIGINAL_BROADCAST_NPDU => Self {
                function: DataLinkFunction::OriginalBroadcastNpdu(npdu),
            },
            Self::BVLC_ORIGINAL_UNICAST_NPDU => Self {
                function: DataLinkFunction::OriginalUnicastNpdu(npdu),
            },
            _ => todo!(),
        };

        Ok(data_link)
    }

    fn update_len(buffer: &mut Writer) {
        let len = buffer.index as u16;
        let src = len.to_be_bytes();
        buffer.buf[2..4].copy_from_slice(&src);
    }
}
