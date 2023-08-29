use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestSerivice},
        services::change_of_value::SubscribeCov,
        unconfirmed::UnconfirmedRequest,
    },
    common::{
        helper::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
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
    let object_id = ObjectId::new(ObjectType::ObjectAnalogInput, 4);
    let cov = SubscribeCov::new(1, object_id, false, 5);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::SubscribeCov(cov));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);

    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));
    let mut buffer = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buffer);
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    // receive reply ack
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::new();
    let message = DataLink::decode(&mut reader, buf);
    println!("Decoded:  {:?}\n", message);

    // receive cov notification
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::new();
    let message = DataLink::decode(&mut reader, buf);
    println!("Decoded:  {:?}\n", message);

    let notification = match message {
        Ok(message) => match message.npdu {
            Some(x) => match x.network_message {
                NetworkMessage::Apdu(apdu) => match apdu {
                    ApplicationPdu::UnconfirmedRequest(UnconfirmedRequest::CovNotification(x)) => {
                        Some(x)
                    }
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        _ => None,
    };

    if let Some(notification) = notification {
        while let Some(property) = notification.decode_next(&mut reader, buf) {
            println!("Value: {:?}", property)
        }
    }

    Ok(())
}
