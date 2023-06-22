#![allow(dead_code, unreachable_code, unused_variables)]

use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::{
            ApplicationPdu, ConfirmedRequest, ConfirmedRequestSerivice, UnconfirmedRequest,
        },
        read_property::ReadProperty,
        who_is::WhoIs,
    },
    common::{
        helper::{Buffer, Reader},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{DestinationAddress, MessagePriority, NetworkMessage, NetworkPdu},
    },
};

// This is a demo application showcasing some of the functionality of this bacnet library

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();

    // broadcast_who_is()
    read_property_list()
}

fn read_property_list() -> Result<(), Error> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    let object_id = ObjectId {
        object_type: ObjectType::ObjectDevice,
        id: 20088,
    };
    let read_property = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let confirmed_request =
        ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadProperty(read_property));
    let apdu = ApplicationPdu::ConfirmedRequest(confirmed_request);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu(npdu));

    let mut buffer = Buffer::new();
    data_link.encode(&mut buffer);

    let buf = buffer.to_bytes();
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    // return Ok(());
    let mut buf = vec![0; 1024];
    loop {
        let (n, peer) = socket.recv_from(&mut buf).unwrap();
        let payload = &buf[..n];
        println!("Received: {:02x?} from {:?}", payload, peer);
        let mut reader = Reader::new(payload);
        let message = DataLink::decode(&mut reader);
        println!("Decoded:  {:?}\n", message);
    }
}

fn broadcast_who_is() -> Result<(), Error> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;
    socket.set_broadcast(true)?;

    let who_is = WhoIs {};
    let apdu = ApplicationPdu::UnconfirmedRequest(UnconfirmedRequest::WhoIs(who_is));
    let src = None;
    let dst = Some(DestinationAddress::new(0xffff, None));
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, false, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalBroadcastNpdu(npdu));

    let mut buffer = Buffer::new();
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
        let mut reader = Reader::new(payload);
        let message = DataLink::decode(&mut reader);
        println!("Decoded:  {:?}\n", message);
    }
}
