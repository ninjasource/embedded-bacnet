use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::{ApplicationPdu, UnconfirmedRequest},
        who_is::WhoIs,
    },
    common::helper::{Buffer, Reader},
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{DestinationAddress, MessagePriority, NetworkMessage, NetworkPdu},
    },
};

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;
    socket.set_broadcast(true)?;

    let who_is = WhoIs {};
    let apdu = ApplicationPdu::UnconfirmedRequest(UnconfirmedRequest::WhoIs(who_is));
    let src = None;
    let dst = Some(DestinationAddress::new(0xffff, None));
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, false, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalBroadcastNpdu(npdu));

    let mut buffer = vec![0; 16 * 1024];
    let mut buffer = Buffer::new(&mut buffer);
    data_link.encode(&mut buffer);

    let buf = buffer.to_bytes();
    let addr = format!("255.255.255.255:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    let mut buf = vec![0; 1024];
    loop {
        let (n, peer) = socket.recv_from(&mut buf).unwrap();
        let payload = &buf[..n];
        println!("Received: {:02x?} from {:?}", payload, peer);
        let mut reader = Reader::new(payload.len());
        let message = DataLink::decode(&mut reader, payload);
        println!("Decoded:  {:?}\n", message);
    }
}
