// cargo run --example change_of_value -- --help

use clap::Parser;
use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestService},
        services::change_of_value::SubscribeCov,
        unconfirmed::UnconfirmedRequest,
    },
    common::{
        io::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
    },
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
    },
};
use std::net::UdpSocket;

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

/// A Bacnet Client example to subscribe to Change-Of-Value events
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address with port e.g. "192.168.1.249:47808"
    #[arg(short, long)]
    addr: String,
}

fn main() -> Result<(), MainError> {
    simple_logger::init().unwrap();
    let args = Args::parse();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectAnalogInput, 4);
    let cov = SubscribeCov::new(1, object_id, false, 5);
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::SubscribeCov(cov));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));
    let mut buffer = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buffer);
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    socket.send_to(buf, &args.addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, &args.addr);

    // receive reply ack
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf)?;
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf);
    println!("Decoded:  {:?}\n", message);

    // receive cov notification
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf)?;
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::default();
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
        for property in &notification.values {
            println!("Value: {:?}", property?)
        }
    }

    Ok(())
}
