use std::net::UdpSocket;

use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestService},
        services::read_property::{ReadProperty, ReadPropertyAck, ReadPropertyValue},
    },
    common::{
        io::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
    },
};

const IP_ADDRESS: &str = "192.168.1.249:47808";
const DEVICE_ID: u32 = 79079;

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
fn main() -> Result<(), MainError> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectDevice, DEVICE_ID);
    let read_property = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::ReadProperty(read_property));
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

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf)?;
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf)?;
    println!("Decoded:  {:?}\n", message);
    let ack: ReadPropertyAck = message.try_into()?;
    match ack.property_value {
        ReadPropertyValue::ObjectIdList(list) => {
            for item in &list {
                println!("{:?}", item);
            }
        }
        _ => {
            // ignore
        }
    }

    Ok(())
}
