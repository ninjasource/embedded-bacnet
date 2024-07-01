// cargo run --example who_is_broadcast
// cargo run --example who_is_broadcast -- --addr "192.168.1.249:47808"

use std::{io::Error, net::UdpSocket};

use clap::Parser;
use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu, services::who_is::WhoIs, unconfirmed::UnconfirmedRequest,
    },
    common::io::{Reader, Writer},
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{DestinationAddress, MessagePriority, NetworkMessage, NetworkPdu},
    },
};

#[derive(Debug)]
pub enum MainError {
    Io(std::io::Error),
    Bacnet(embedded_bacnet::common::error::Error),
}

impl From<std::io::Error> for MainError {
    fn from(value: std::io::Error) -> Self {
        MainError::Io(value)
    }
}

impl From<embedded_bacnet::common::error::Error> for MainError {
    fn from(value: embedded_bacnet::common::error::Error) -> Self {
        MainError::Bacnet(value)
    }
}

/// A Bacnet Client example to send a who_is request and wait from an i_am reply.
/// NOTE: this example works with broadcast UDP packets by default (255.255.255.255) which may be blocked by your network
/// You can get around this by sending the who_is directly to a known IP address
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address with port e.g. "192.168.1.249:47808"
    #[arg(short, long, default_value = "255.255.255.255:47808")]
    addr: String,
}

// NOTE: this example works with broadcast UDP packets which may be blocked by your network
// You can get around this by sending the who_is directly to a known IP aaddress
// Since sending who_is requests are somewhat network specific (you need to know who the peer is) we don't have a conveneince function for it as this would be messy.
fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();
    let args = Args::parse();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC1))?;
    socket.set_broadcast(true)?;

    let who_is = WhoIs {};
    let apdu = ApplicationPdu::UnconfirmedRequest(UnconfirmedRequest::WhoIs(who_is));
    let dst = Some(DestinationAddress::new(0xffff, None));
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(None, dst, false, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalBroadcastNpdu, Some(npdu));

    let mut buffer = vec![0; 1500];

    {
        let mut buffer = Writer::new(&mut buffer);
        data_link.encode(&mut buffer);
        let buf = buffer.to_bytes();
        socket.send_to(buf, &args.addr)?;
        println!("Sent:     {:02x?} to {}\n", buf, &args.addr);
    }

    loop {
        let (n, peer) = socket.recv_from(&mut buffer)?;
        let payload = &buffer[..n];
        println!("Received: {:02x?} from {:?}", payload, peer);
        let mut reader = Reader::default();
        let message = DataLink::decode(&mut reader, payload);
        println!("Decoded:  {:?}\n", message);
    }
}
