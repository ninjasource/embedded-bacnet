// cargo run --example update_schedule -- --addr "192.168.1.249:47808"

use std::net::UdpSocket;

use clap::Parser;
use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestService},
        primitives::data_value::{ApplicationDataValue, ApplicationDataValueWrite},
        services::{
            read_property_multiple::{
                PropertyValue, ReadPropertyMultiple, ReadPropertyMultipleAck,
                ReadPropertyMultipleObject,
            },
            write_property::WriteProperty,
        },
    },
    common::{
        daily_schedule::WeeklySchedule,
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

/// A Bacnet Client example to update a schedule
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
    let object_id = ObjectId::new(ObjectType::ObjectSchedule, 1);
    let property_ids = [PropertyId::PropObjectName, PropertyId::PropWeeklySchedule];
    let rpm = ReadPropertyMultipleObject::new(object_id, &property_ids);
    let objects = [rpm];
    let rpm = ReadPropertyMultiple::new(&objects);
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::ReadPropertyMultiple(rpm));
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

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf)?;
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf)?;
    println!("Decoded: {:?}", message);

    let mut monday = vec![];
    let mut tuesday = vec![];
    let mut wednesday = vec![];
    let mut thursday = vec![];
    let mut friday = vec![];
    let mut saturday = vec![];
    let mut sunday = vec![];

    let message: ReadPropertyMultipleAck = message.try_into()?;

    for values in &message {
        let values = values?;
        for x in values.property_results.into_iter() {
            let x = x?;
            match x.value {
                PropertyValue::PropValue(ApplicationDataValue::WeeklySchedule(weekly_schedule)) => {
                    monday = weekly_schedule
                        .monday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    tuesday = weekly_schedule
                        .tuesday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    wednesday = weekly_schedule
                        .wednesday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    thursday = weekly_schedule
                        .thursday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    friday = weekly_schedule
                        .friday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    saturday = weekly_schedule
                        .saturday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    sunday = weekly_schedule
                        .sunday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                }
                _ => {
                    // do nothing
                }
            }
        }
    }

    // change the schedule
    monday[0].time.hour = 8;

    let weekly_schedule = WeeklySchedule::new(
        &monday, &tuesday, &wednesday, &thursday, &friday, &saturday, &sunday,
    );

    println!("{:?}", weekly_schedule);

    // encode packet
    let write_property = WriteProperty::new(
        ObjectId::new(ObjectType::ObjectSchedule, 1),
        PropertyId::PropWeeklySchedule,
        None,
        None,
        ApplicationDataValueWrite::WeeklySchedule(weekly_schedule),
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
    socket.send_to(buf, &args.addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, &args.addr);

    // receive reply ack
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf);
    println!("Decoded:  {:?}\n", message);

    Ok(())
}
