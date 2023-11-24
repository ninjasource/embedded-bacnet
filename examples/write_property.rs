use std::net::UdpSocket;

use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestService},
        primitives::data_value::{ApplicationDataValueWrite, Enumerated},
        services::write_property::WriteProperty,
    },
    common::{
        io::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
        spec::Binary,
    },
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
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

const IP_ADDRESS: &str = "192.168.1.249:47808";

fn main() -> Result<(), MainError> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC1))?;

    // encode packet
    let write_property = WriteProperty::new(
        ObjectId::new(ObjectType::ObjectBinaryValue, 3),
        PropertyId::PropPresentValue,
        None,
        None,
        ApplicationDataValueWrite::Enumerated(Enumerated::Binary(Binary::On)),
    );
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::WriteProperty(write_property));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));
    let mut buffer = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buffer);
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    socket.send_to(buf, IP_ADDRESS)?;
    println!("Sent:     {:02x?} to {}\n", buf, IP_ADDRESS);

    // receive reply ack
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf)?;
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf)?;
    println!("Decoded:  {:?}\n", message);

    Ok(())
}
