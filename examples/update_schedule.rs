use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestSerivice},
        primitives::data_value::ApplicationDataValue,
        services::{
            read_property_multiple::{
                PropertyValue, ReadPropertyMultiple, ReadPropertyMultipleObject,
            },
            write_property::WriteProperty,
        },
    },
    common::{
        daily_schedule::WeeklySchedule,
        helper::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
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
    let object_id = ObjectId::new(ObjectType::ObjectSchedule, 1);
    let property_ids = [PropertyId::PropObjectName, PropertyId::PropWeeklySchedule];
    let rpm = ReadPropertyMultipleObject::new(object_id, &property_ids);
    let objects = [rpm];
    let rpm = ReadPropertyMultiple::new(&objects);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadPropertyMultiple(rpm));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu(npdu));
    let mut buffer = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buffer);
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::new();
    let message = DataLink::decode(&mut reader, buf).unwrap();

    let mut schedule = vec![];

    if let Some(message) = message.get_read_property_multiple_ack() {
        while let Some(values) = message.decode_next(&mut reader, buf) {
            while let Some(x) = values.decode_next(&mut reader, buf) {
                match x.value {
                    PropertyValue::PropValue(ApplicationDataValue::WeeklySchedule(
                        weekly_schedule,
                    )) => {
                        let mut weekly_schedule_reader = weekly_schedule.decode();
                        while let Some(day_time_value) =
                            weekly_schedule_reader.decode_next(&mut reader, buf)
                        {
                            schedule.push(day_time_value);
                        }
                    }
                    _ => {
                        // do nothing
                    }
                }
            }
        }
    }

    println!("{schedule:?}");

    let mut monday = vec![];
    let mut tuesday = vec![];
    let mut wednesday = vec![];
    let mut thursday = vec![];
    let mut friday = vec![];
    let mut saturday = vec![];
    let mut sunday = vec![];

    for item in schedule {
        match item.day_of_week {
            0 => monday.push(item.time_value),
            1 => tuesday.push(item.time_value),
            2 => wednesday.push(item.time_value),
            3 => thursday.push(item.time_value),
            4 => friday.push(item.time_value),
            5 => saturday.push(item.time_value),
            6 => sunday.push(item.time_value),
            _ => unreachable!(),
        }
    }
    // change the schedule
    monday[0].time.hour = 7;

    let mut weekly_schedule = WeeklySchedule::new();
    weekly_schedule.monday = &monday;
    weekly_schedule.tuesday = &tuesday;
    weekly_schedule.wednesday = &wednesday;
    weekly_schedule.thursday = &thursday;
    weekly_schedule.friday = &friday;
    weekly_schedule.saturday = &saturday;
    weekly_schedule.sunday = &sunday;

    // encode packet
    let write_property = WriteProperty::new(
        ObjectId::new(ObjectType::ObjectSchedule, 1),
        PropertyId::PropWeeklySchedule,
        None,
        None,
        ApplicationDataValue::WeeklySchedule(weekly_schedule),
    );
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::WriteProperty(write_property));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu(npdu));
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

    Ok(())
}
