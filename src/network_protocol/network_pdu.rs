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

#[derive(Debug, Copy, Clone)]
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
        })
    }
}

/// Opaque device address.
///
/// The format of the address depends on the network for which the address is valid.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceAddress {
    len: u8,
    data: [u8; Self::MAX_LEN],
}

#[deprecated(note = "use DeviceAddress")]
pub type Addr = DeviceAddress;

pub type SourceAddress = NetworkAddress;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NetworkAddress {
    pub net: u16,
    pub addr: DeviceAddress,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DestinationAddress {
    pub network_address: NetworkAddress,
    pub hop_count: u8,
}

impl DestinationAddress {
    pub fn new(net: u16, addr: Option<DeviceAddress>) -> Self {
        let addr = addr.unwrap_or(DeviceAddress::BROADCAST);
        Self {
            network_address: NetworkAddress { net, addr },
            hop_count: 255,
        }
    }
}

impl NetworkAddress {
    pub fn encode(&self, writer: &mut Writer) {
        writer.extend_from_slice(&self.net.to_be_bytes());
        self.addr.encode(writer);
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let net = u16::from_be_bytes(reader.read_bytes(buf)?);
        let addr = DeviceAddress::decode(reader, buf)?;
        Ok(Self { net, addr })
    }
}

impl DeviceAddress {
    /// The maximum length of an address.
    const MAX_LEN: usize = 18; // IPv6 address + port number

    /// The broadcast address.
    ///
    /// Used to broadcast a message to all devices on a remote network.
    pub const BROADCAST: Self = Self {
        len: 0,
        data: [0; Self::MAX_LEN],
    };

    /// Make a new address from a byte slice.
    pub fn new(value: &[u8]) -> Result<Self, Error> {
        if value.len() > Self::MAX_LEN {
            Err(Error::Length((
                "network address too long: maximum length is 18 bytes",
                value.len() as u32,
            )))
        } else {
            let mut data = [0; Self::MAX_LEN];
            data[..value.len()].copy_from_slice(value);
            Ok(Self {
                len: value.len() as u8,
                data,
            })
        }
    }

    /// Make an address from a byte array.
    ///
    /// Fails to compile if N > Self::MAX_LEN.
    pub const fn from_array<const N: usize>(input: [u8; N]) -> Self {
        const {
            assert!(N <= Self::MAX_LEN);
        }
        let mut data = [0u8; Self::MAX_LEN];
        let mut i = 0;
        while i < N {
            data[i] = input[i];
            i += 1;
        }
        Self { len: N as u8, data }
    }

    /// Make a new BACnet/IP IPv4 address (4 byte IP address and 2 byte port number).
    pub const fn new_ipv4(addr: core::net::SocketAddrV4) -> Self {
        let ip = addr.ip().octets();
        let port = addr.port().to_be_bytes();

        Self::from_array([ip[0], ip[1], ip[2], ip[3], port[0], port[1]])
    }

    /// Make a new BACnet/IP IPv6 address (16 byte IP address and 2 byte port number).
    pub const fn new_ipv6(addr: core::net::SocketAddrV6) -> Self {
        let ip = addr.ip().octets();
        let port = addr.port().to_be_bytes();

        Self::from_array([
            ip[0], ip[1], ip[2], ip[3], ip[4], ip[5], ip[6], ip[7], ip[8], ip[9], ip[10], ip[11],
            ip[12], ip[13], ip[14], ip[15], port[0], port[1],
        ])
    }

    /// Make a new MS/TP address (a single byte).
    pub const fn new_mstp(addr: u8) -> Self {
        Self::from_array([addr])
    }

    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_broadcast(&self) -> bool {
        self.is_empty()
    }

    pub fn data(&self) -> &[u8] {
        let len = self.len as usize;
        &self.data[..len]
    }

    pub fn encode(&self, writer: &mut Writer) {
        writer.push(self.len);
        writer.extend_from_slice(self.data());
    }

    pub fn decode(reader: &mut Reader, buf: &[u8]) -> Result<Self, Error> {
        let len = reader.read_byte(buf)?;
        let data = reader.read_slice(len.into(), buf)?;
        Self::new(data)
    }
}

impl TryFrom<&[u8]> for DeviceAddress {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<core::net::SocketAddrV4> for DeviceAddress {
    fn from(value: core::net::SocketAddrV4) -> Self {
        Self::new_ipv4(value)
    }
}

impl From<core::net::SocketAddrV6> for DeviceAddress {
    fn from(value: core::net::SocketAddrV6) -> Self {
        Self::new_ipv6(value)
    }
}

impl From<core::net::SocketAddr> for DeviceAddress {
    fn from(value: core::net::SocketAddr) -> Self {
        match value {
            core::net::SocketAddr::V4(x) => x.into(),
            core::net::SocketAddr::V6(x) => x.into(),
        }
    }
}

impl TryFrom<DeviceAddress> for core::net::SocketAddrV4 {
    type Error = Error;

    fn try_from(value: DeviceAddress) -> Result<Self, Self::Error> {
        match value.data() {
            &[a, b, c, d, port_high, port_low] => {
                let ip = core::net::Ipv4Addr::from_octets([a, b, c, d]);
                let port = u16::from_be_bytes([port_high, port_low]);
                Ok(Self::new(ip, port))
            }
            _ => Err(Error::Length((
                "BACnet/IPv4 addresses must be 6 byes long",
                value.len().into(),
            ))),
        }
    }
}

impl TryFrom<DeviceAddress> for core::net::SocketAddrV6 {
    type Error = Error;

    fn try_from(value: DeviceAddress) -> Result<Self, Self::Error> {
        match value.data() {
            &[b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, port_high, port_low] =>
            {
                let ip = core::net::Ipv6Addr::from_octets([
                    b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15,
                ]);
                let port = u16::from_be_bytes([port_high, port_low]);
                Ok(Self::new(ip, port, 0, 0))
            }
            _ => Err(Error::Length((
                "BACnet/IPv6 addresses must be 18 byes long",
                value.len().into(),
            ))),
        }
    }
}

impl TryFrom<DeviceAddress> for core::net::SocketAddr {
    type Error = Error;

    fn try_from(value: DeviceAddress) -> Result<Self, Self::Error> {
        if let Ok(v4) = value.try_into() {
            Ok(Self::V4(v4))
        } else if let Ok(v6) = value.try_into() {
            Ok(Self::V6(v6))
        } else {
            Err(Error::Length((
                "BACnet/IP addresses must be 6 or 18 byes long",
                value.len().into(),
            )))
        }
    }
}
