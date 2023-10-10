use std::{io::Error, net::UdpSocket};

use chrono::Datelike;
use chrono::Local;
use chrono::Timelike;
use embedded_bacnet::application_protocol::primitives::data_value::Time;
use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu, primitives::data_value::Date,
        services::time_synchronization::TimeSynchronization, unconfirmed::UnconfirmedRequest,
    },
    common::io::Writer,
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
    },
};

const IP_ADDRESS: &str = "192.168.1.249:47808";

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

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
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, false, MessagePriority::Normal, message);
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
