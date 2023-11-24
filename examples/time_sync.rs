use std::{io::Error, net::UdpSocket};

use chrono::{Datelike, Local, Timelike};
use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestService},
        primitives::data_value::{Date, Time},
        services::{
            read_property_multiple::{
                ReadPropertyMultiple, ReadPropertyMultipleAck, ReadPropertyMultipleObject,
            },
            time_synchronization::TimeSynchronization,
        },
        unconfirmed::UnconfirmedRequest,
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

const IP_ADDRESS: &str = "192.168.1.249:47808";
const DEVICE_ID: u32 = 79079;

fn main() -> Result<(), MainError> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    set_time(&socket)?;
    request_date_time(&socket)?;
    read_date_time(&socket)?;

    Ok(())
}

fn set_time(socket: &UdpSocket) -> Result<(), MainError> {
    let now = Local::now();
    let wday = now.weekday().num_days_from_sunday() as u8; // sunday = 0

    // encode packet
    let date = Date {
        year: now.year() as u16,
        month: now.month() as u8,
        day: now.day() as u8,
        wday,
    };
    let time = Time {
        hour: now.hour() as u8,
        minute: now.minute() as u8,
        second: 0,
        hundredths: 0,
    };
    let time_sync = TimeSynchronization { date, time };
    let apdu =
        ApplicationPdu::UnconfirmedRequest(UnconfirmedRequest::TimeSynchronization(time_sync));
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(None, None, false, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));
    let mut buffer = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buffer);
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    socket.send_to(buf, IP_ADDRESS)?;
    println!("Sent:     {:02x?} to {}\n", buf, IP_ADDRESS);
    Ok(())
}

fn request_date_time(socket: &UdpSocket) -> Result<(), Error> {
    println!("Fetching date time");

    let object_id = ObjectId::new(ObjectType::ObjectDevice, DEVICE_ID);
    let property_ids = [PropertyId::PropLocalDate, PropertyId::PropLocalTime];
    let rpm = ReadPropertyMultipleObject::new(object_id, &property_ids);
    let objects = [rpm];
    let rpm = ReadPropertyMultiple::new(&objects);
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::ReadPropertyMultiple(rpm));
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
    Ok(())
}

pub fn read_date_time(socket: &UdpSocket) -> Result<(), MainError> {
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
