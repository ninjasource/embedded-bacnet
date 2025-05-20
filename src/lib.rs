#![cfg_attr(not(test), no_std)]
#![allow(clippy::large_enum_variant)]

// This library supports the IP version of bacnet and this is how the network packet is wrapped:
//
// UdpPacket
// |
// -> DataLink
//    |
//    -> NetworkPdu
//       |
//       -> ApplicationPdu
//
// NOTE: Pdu stands for Protocol Data Unit
// The starting point for using this library is DataLink::new()

// Network Layer and Data Link Layer
pub mod network_protocol;

// Application Layer
pub mod application_protocol;

// Common functions and constants
pub mod common;

pub mod simple;

#[cfg(feature = "alloc")]
extern crate alloc;
