// cargo run --example read_range -- --addr "192.168.1.249:47808"

use core::ops::Range;
use std::net::UdpSocket;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use clap::Parser;
use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ComplexAck, ComplexAckService, ConfirmedRequest, ConfirmedRequestService},
        primitives::data_value::ApplicationDataValue,
        services::{
            read_property::{ReadProperty, ReadPropertyAck, ReadPropertyValue},
            read_range::{ReadRange, ReadRangeByPosition, ReadRangeRequestType, ReadRangeValue},
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

/// A Bacnet Client example to read a range of values from trend log #4 (typically used for displaying a chart)
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
    let object_id = ObjectId::new(ObjectType::ObjectTrendlog, 4);

    let record_count = get_record_count(&args.addr, &socket, object_id.clone())? as usize;
    println!("Record count {record_count}");

    const MAX_LOG_COUNT_PER_REQ: usize = 55;

    for row in (1..=record_count).step_by(MAX_LOG_COUNT_PER_REQ) {
        get_items_for_range(
            &args.addr,
            &socket,
            object_id.clone(),
            row..MAX_LOG_COUNT_PER_REQ,
        )?;
    }

    Ok(())
}

fn get_items_for_range(
    addr: &str,
    socket: &UdpSocket,
    object_id: ObjectId,
    range: Range<usize>,
) -> Result<(), MainError> {
    // encode packet
    let request_type = ReadRangeRequestType::ByPosition(ReadRangeByPosition {
        index: range.start as u32,
        count: range.end as u32,
    });
    let rp = ReadRange::new(object_id, PropertyId::PropLogBuffer, request_type);
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::ReadRange(rp));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));
    let mut buffer = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buffer);
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    socket.send_to(buf, addr)?;

    // receive reply
    let mut buf = vec![0; 4096];
    let (n, _peer) = socket.recv_from(&mut buf)?;
    let buf = &buf[..n];
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf)?;
    let ack: ComplexAck = message.try_into()?;

    match ack.service {
        ComplexAckService::ReadRange(read_range) => {
            for item in &read_range.item_data {
                let item = item?;
                let value = match item.value {
                    ReadRangeValue::Real(x) => x,
                    _ => 0.0,
                };
                let date_time = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        item.date.year as i32,
                        item.date.month as u32,
                        item.date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(item.time.hour as u32, item.time.minute as u32, 0)
                        .unwrap(),
                );

                println!("{} {}", date_time, value);
            }
        }
        _ => {
            // do nothing
        }
    }

    Ok(())
}

fn get_record_count(addr: &str, socket: &UdpSocket, object_id: ObjectId) -> Result<u32, MainError> {
    // encode packet
    let rp = ReadProperty::new(object_id, PropertyId::PropRecordCount);
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::ReadProperty(rp));
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
    socket.send_to(buf, addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf)?;
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf)?;
    println!("Decoded:  {:?}\n", message);
    let message: ReadPropertyAck = message.try_into()?;

    // read values
    if let ReadPropertyValue::ApplicationDataValue(ApplicationDataValue::UnsignedInt(x)) =
        message.property_value
    {
        Ok(x)
    } else {
        Ok(0)
    }
}
