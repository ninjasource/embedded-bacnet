use crate::{
    application_protocol::application_pdu::ApplicationPdu,
    common::{
        error::Error,
        io::{Reader, Writer},
    },
};

// Network Layer Protocol Data Unit
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NetworkPdu<'a> {
    pub src: Option<SourceAddress>,
    pub dst: Option<DestinationAddress>,
    pub expect_reply: bool,
    pub message_priority: MessagePriority,
    pub network_message: NetworkMessage<'a>,

    pub raw_payload: &'a [u8],
}

// NOTE: this is actually a control flag
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum MessagePriority {
    Normal = 0,
    Urgent = 1,
    CriticalEquipment = 2,
    LifeSafety = 3,
}

impl From<u8> for MessagePriority {
    fn from(value: u8) -> Self {
        const MASK: u8 = 0b0000_0011;
        let value = value & MASK;

        match value {
            0 => MessagePriority::Normal,
            1 => MessagePriority::Urgent,
            2 => MessagePriority::CriticalEquipment,
            3 => MessagePriority::LifeSafety,
            _ => unreachable!(), // because of mask
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
enum ControlFlags {
    NetworkLayerMessage = 1 << 7,
    HasDestination = 1 << 5,
    HasSource = 1 << 3,
    ExpectingReply = 1 << 2,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NetworkMessage<'a> {
    Apdu(ApplicationPdu<'a>),
    MessageType(MessageType),
    CustomMessageType(u8),
}

// Network Layer Message Type
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum MessageType {
    WhoIsRouterToNetwork = 0,
    IAmRouterToNetwork = 1,
    ICouldBeRouterToNetwork = 2,
    RejectMessageToNetwork = 3,
    RouterBusyToNetwork = 4,
    RouterAvailableToNetwork = 5,
    InitRtTable = 6,
    InitRtTableAck = 7,
    EstablishConnectionToNetwork = 8,
    DisconnectConnectionToNetwork = 9,
    ChallengeRequest = 10,
    SecurityPayload = 11,
    SecurityResponse = 12,
    RequestKeyUpdate = 13,
    UpdateKeySet = 14,
    UpdateDistributionKey = 15,
    RequestMasterKey = 16,
    SetMasterKey = 17,
    WhatIsNetworkNumber = 18,
    NetworkNumberIs = 19,
    // X'14' to X'7F': Reserved for use by ASHRAE
    // X'80' to X'FF': Available for vendor proprietary messages
}

impl TryFrom<u8> for MessageType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::WhoIsRouterToNetwork),
            1 => Ok(Self::IAmRouterToNetwork),
            2 => Ok(Self::ICouldBeRouterToNetwork),
            3 => Ok(Self::RejectMessageToNetwork),
            4 => Ok(Self::RouterBusyToNetwork),
            5 => Ok(Self::RouterAvailableToNetwork),
            6 => Ok(Self::InitRtTable),
            7 => Ok(Self::InitRtTableAck),
            8 => Ok(Self::EstablishConnectionToNetwork),
            9 => Ok(Self::DisconnectConnectionToNetwork),
            10 => Ok(Self::ChallengeRequest),
            11 => Ok(Self::SecurityPayload),
            12 => Ok(Self::SecurityResponse),
            13 => Ok(Self::RequestKeyUpdate),
            14 => Ok(Self::UpdateKeySet),
            15 => Ok(Self::UpdateDistributionKey),
            16 => Ok(Self::RequestMasterKey),
            17 => Ok(Self::SetMasterKey),
            18 => Ok(Self::WhatIsNetworkNumber),
            19 => Ok(Self::NetworkNumberIs),
            _ => Err(value),
        }
    }
}

impl<'a> NetworkPdu<'a> {
    const VERSION: u8 = 0x01; // ASHRAE 135-1995
    pub fn new(
        src: Option<SourceAddress>,
        dst: Option<DestinationAddress>,
        expect_reply: bool,
        message_priority: MessagePriority,
        message: NetworkMessage<'a>,
    ) -> Self {
        Self {
            src,
            dst,
            expect_reply,
            message_priority,
            network_message: message,
            raw_payload: &[], // empty unless decoding
        }
    }

    pub fn encode(&self, writer: &mut Writer) {
        writer.push(Self::VERSION);
        writer.push(self.calculate_control());

        if let Some(dst) = self.dst.as_ref() {
            dst.network_address.encode(writer);
        }

        if let Some(src) = self.src.as_ref() {
            src.encode(writer);
        }

        // hop count comes after src
        if let Some(dst) = self.dst.as_ref() {
            writer.push(dst.hop_count);
        }

        match &self.network_message {
            NetworkMessage::Apdu(adpu) => adpu.encode(writer),
            NetworkMessage::MessageType(message_type) => {
                writer.push(message_type.clone() as u8);
            }
            NetworkMessage::CustomMessageType(message_type) => {
                writer.push(*message_type);
            }
        };
    }

    fn calculate_control(&self) -> u8 {
        let is_network_layer_message = match &self.network_message {
            NetworkMessage::Apdu(_) => 0,
            NetworkMessage::MessageType(_) => ControlFlags::NetworkLayerMessage as u8,
            NetworkMessage::CustomMessageType(_) => ControlFlags::NetworkLayerMessage as u8,
        };

        let has_destination = match self.dst.as_ref() {
            Some(dst) => {
                if dst.network_address.net > 0 {
                    ControlFlags::HasDestination as u8
                } else {
                    0
                }
            }
            None => 0,
        };

        let has_source = match self.src.as_ref() {
            Some(src) => {
                if src.net > 0 && src.net != 0xFFFF {
                    ControlFlags::HasSource as u8
                } else {
                    0
                }
            }
            None => 0,
        };
        let expecting_reply = if self.expect_reply {
            ControlFlags::ExpectingReply as u8
        } else {
            0
        };
        let message_priority = self.message_priority.clone() as u8;

        is_network_layer_message | has_destination | has_source | expecting_reply | message_priority
    }

    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        // ignore version
        let _version = reader.read_byte(buf)?;

        // read and decode control byte
        let control = reader.read_byte(buf)?;
        let has_dst = (control & ControlFlags::HasDestination as u8) > 0;
        let has_src = (control & ControlFlags::HasSource as u8) > 0;
        let is_network_message = (control & ControlFlags::NetworkLayerMessage as u8) > 0;
        let expect_reply = (control & ControlFlags::ExpectingReply as u8) > 0;
        let message_priority: MessagePriority = control.into();

        let dst = if has_dst {
            Some(NetworkAddress::decode(reader, buf)?)
        } else {
            None
        };

        let src = if has_src {
            Some(NetworkAddress::decode(reader, buf)?)
        } else {
            None
        };

        // if dst exists then read the hop_count (it comes after src for some reason)
        let dst = if let Some(dst) = dst {
            let hop_count = reader.read_byte(buf)?;
            Some(DestinationAddress {
                network_address: dst,
                hop_count,
            })
        } else {
            None
        };

        let payload_start_index = reader.index;
        let network_message = if is_network_message {
            let message_type = reader.read_byte(buf)?;
            match message_type.try_into() {
                Ok(message_type) => NetworkMessage::MessageType(message_type),
                Err(custom_message_type) => NetworkMessage::CustomMessageType(custom_message_type),
            }
        } else {
            let apdu = ApplicationPdu::decode(reader, buf)?;
            NetworkMessage::Apdu(apdu)
        };

        Ok(Self {
            dst,
            src,
            expect_reply,
            message_priority,
            network_message,
            raw_payload: &buf[payload_start_index..],
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ipv4Addr {
    pub addr: [u8; 4],
    pub port: u16,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Addr {
    Ipv4(Ipv4Addr),
    Mac(u8),
}

impl Addr {
    pub fn new_ipv4(addr: [u8; 4], port: u16) -> Self {
        Self::Ipv4(Ipv4Addr { addr, port })
    }
}

const IPV4_ADDR_LEN: u8 = 6;

pub type SourceAddress = NetworkAddress;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NetworkAddress {
    pub net: u16,
    pub addr: Option<Addr>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DestinationAddress {
    pub network_address: NetworkAddress,
    pub hop_count: u8,
}

impl DestinationAddress {
    pub fn new(net: u16, addr: Option<Addr>) -> Self {
        Self {
            network_address: NetworkAddress { net, addr },
            hop_count: 255,
        }
    }
}

impl NetworkAddress {
    pub fn encode(&self, writer: &mut Writer) {
        writer.extend_from_slice(&self.net.to_be_bytes());
        match self.addr.as_ref() {
            Some(addr) => {
                match addr {
                    Addr::Mac(mac) => {
                        let encoded = &mac.to_be_bytes();
                        writer.push(encoded.len() as u8);
                        writer.extend_from_slice(encoded);
                    },
                    Addr::Ipv4(addr) => {
                        writer.push(IPV4_ADDR_LEN);
                        writer.extend_from_slice(&addr.addr);
                        writer.extend_from_slice(&addr.port.to_be_bytes());
                    }
                }
            }
            None => writer.push(0),
        }
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let net = u16::from_be_bytes(reader.read_bytes(buf)?);
        let len = reader.read_byte(buf)?;
        match len {
            IPV4_ADDR_LEN => {
                let ipv4: [u8; 4] = reader.read_bytes(buf)?;
                let port = u16::from_be_bytes(reader.read_bytes(buf)?);

                Ok(Self {
                    net,
                    addr: Some(Addr::Ipv4(Ipv4Addr { port, addr: ipv4 })),
                })
            }
            0 => Ok(Self { net, addr: None }),
            x => Err(Error::Length((
                "NetworkAddress decode ip len can only be 6 or 0",
                x as u32,
            ))),
        }
    }
}
