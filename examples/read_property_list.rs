use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ComplexAckService, ConfirmedRequest, ConfirmedRequestSerivice},
        services::read_property::{ReadProperty, ReadPropertyValue},
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

//const IP_ADDRESS: &str = "192.168.1.215:47808";
//const DEVICE_ID: u32 = 76011;
const IP_ADDRESS: &str = "192.168.1.249:47808";
const DEVICE_ID: u32 = 79079;

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectDevice, DEVICE_ID);
    let read_property = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadProperty(read_property));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
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
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::new();
    let message = DataLink::decode(&mut reader, buf);
    println!("Decoded:  {:?}\n", message);

    let network_message = message.unwrap().npdu.unwrap().network_message;
    if let NetworkMessage::Apdu(ApplicationPdu::ComplexAck(apdu)) = network_message {
        match apdu.service {
            ComplexAckService::ReadProperty(x) => match x.property_value {
                ReadPropertyValue::ObjectIdList(list) => {
                    for item in list {
                        println!("{:?}", item);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(())
}
