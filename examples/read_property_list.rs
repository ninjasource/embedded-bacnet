use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::{ApplicationPdu, ConfirmedRequest, ConfirmedRequestSerivice},
        read_property::ReadProperty,
    },
    common::{
        helper::{Buffer, Reader},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
    },
};

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectDevice, 20088);
    let read_property = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadProperty(read_property));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu(npdu));
    let mut buffer = Buffer::new();
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let payload = &buf[..n];
    println!("Received: {:02x?} from {:?}", payload, peer);
    let mut reader = Reader::new(payload);
    let message = DataLink::decode(&mut reader);
    println!("Decoded:  {:?}\n", message);

    Ok(())
}
