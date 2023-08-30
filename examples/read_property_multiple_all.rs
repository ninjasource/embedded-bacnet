use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestSerivice},
        services::read_property_multiple::{ReadPropertyMultiple, ReadPropertyMultipleObject},
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
const IP_ADDRESS: &str = "192.168.1.249:47808";

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectAnalogInput, 1);
    let rpm = ReadPropertyMultipleObject::new(object_id, &[PropertyId::PropAll]);
    let objects = [rpm];
    let rpm = ReadPropertyMultiple::new(&objects);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadPropertyMultiple(rpm));
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
    let message = DataLink::decode(&mut reader, buf).unwrap();
    println!("Decoded:  {:?}\n", message);

    // read values
    if let Some(message) = message.get_read_property_multiple_ack_into() {
        for values in message {
            for x in values.into_iter() {
                println!("{:?}", x);
            }
        }
    }

    Ok(())
}
