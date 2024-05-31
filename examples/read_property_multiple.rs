// cargo run --example read_property_multiple -- --addr "192.168.1.249:47808"

mod common;

use std::net::UdpSocket;

use clap::Parser;
use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestService},
        services::read_property_multiple::{
            ReadPropertyMultiple, ReadPropertyMultipleAck, ReadPropertyMultipleObject,
        },
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

/// A Bacnet Client example to read specific property values for analog input #1
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
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC1))?;

    let _server = common::run_dummy_server();

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectAnalogInput, 1);
    let property_ids = [
        PropertyId::PropObjectName,
        PropertyId::PropPresentValue,
        PropertyId::PropUnits,
        PropertyId::PropStatusFlags,
    ];
    let objects = [ReadPropertyMultipleObject::new(object_id, &property_ids)];
    let service =
        ConfirmedRequestService::ReadPropertyMultiple(ReadPropertyMultiple::new(&objects));
    let apdu = ApplicationPdu::ConfirmedRequest(ConfirmedRequest::new(0, service));
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

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf)?;
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf)?;
    println!("Decoded:  {:?}\n", message);
    let message: ReadPropertyMultipleAck = message.try_into()?;

    // read values
    for values in &message {
        let values = values?;
        for x in &values.property_results {
            println!("{:?}", x);
        }
    }

    Ok(())
}
