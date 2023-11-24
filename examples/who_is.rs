use std::{io::Error, net::UdpSocket};

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
enum MainError {
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

// NOTE: this example works with broadcast UDP packets which may be blocked by your network
// You can get around this by sending the who_is directly to a known IP aaddress
fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;
    socket.set_broadcast(true)?;

    let who_is = WhoIs {};
    let apdu = ApplicationPdu::UnconfirmedRequest(UnconfirmedRequest::WhoIs(who_is));
    let dst = Some(DestinationAddress::new(0xffff, None));
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(None, dst, false, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalBroadcastNpdu, Some(npdu));

    let mut buffer = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buffer);
    data_link.encode(&mut buffer);

    let buf = buffer.to_bytes();
    let addr = format!("255.255.255.255:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    let mut buf = vec![0; 1024];
    loop {
        let (n, peer) = socket.recv_from(&mut buf)?;
        let payload = &buf[..n];
        println!("Received: {:02x?} from {:?}", payload, peer);
        let mut reader = Reader::default();
        let message = DataLink::decode(&mut reader, payload);
        println!("Decoded:  {:?}\n", message);
    }
}
