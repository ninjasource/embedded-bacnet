use crate::common::helper::{Buffer, Reader};

use super::network_pdu::NetworkPdu;

// Bacnet Virtual Link Control
#[derive(Debug)]
pub struct DataLink {
    pub function: DataLinkFunction,
}

#[derive(Debug)]
pub enum DataLinkFunction {
    OriginalBroadcastNpdu(NetworkPdu),
    OriginalUnicastNpdu(NetworkPdu),
}

impl DataLink {
    const BVLL_TYPE_BACNET_IP: u8 = 0x81;
    const BVLC_ORIGINAL_UNICAST_NPDU: u8 = 10;
    const BVLC_ORIGINAL_BROADCAST_NPDU: u8 = 11;

    pub fn new(function: DataLinkFunction) -> Self {
        Self { function }
    }

    pub fn encode(&self, buffer: &mut Buffer) {
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

    pub fn decode(reader: &mut Reader) -> Self {
        let bvll_type = reader.read_byte();
        if bvll_type != Self::BVLL_TYPE_BACNET_IP {
            panic!("only BACNET_IP supported");
        }

        let npdu_type = reader.read_byte();
        let _len: u16 = u16::from_be_bytes(reader.read_bytes());
        let npdu = NetworkPdu::decode(reader);

        match npdu_type {
            Self::BVLC_ORIGINAL_BROADCAST_NPDU => Self {
                function: DataLinkFunction::OriginalBroadcastNpdu(npdu),
            },
            Self::BVLC_ORIGINAL_UNICAST_NPDU => Self {
                function: DataLinkFunction::OriginalUnicastNpdu(npdu),
            },
            _ => todo!(),
        }
    }

    fn update_len(buffer: &mut Buffer) {
        let len = buffer.buf.len() as u16;
        let src = len.to_be_bytes();
        buffer.buf[2..4].copy_from_slice(&src);
    }
}
