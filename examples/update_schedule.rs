use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ConfirmedRequest, ConfirmedRequestService},
        primitives::data_value::{ApplicationDataValue, ApplicationDataValueWrite},
        services::{
            read_property_multiple::{
                PropertyValue, ReadPropertyMultiple, ReadPropertyMultipleObject,
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

const IP_ADDRESS: &str = "192.168.1.249:47808";

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectSchedule, 1);
    let property_ids = [PropertyId::PropObjectName, PropertyId::PropWeeklySchedule];
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

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::new();
    let message = DataLink::decode(&mut reader, buf).unwrap();
    println!("Decoded: {:?}", message);

    let mut monday = vec![];
    let mut tuesday = vec![];
    let mut wednesday = vec![];
    let mut thursday = vec![];
    let mut friday = vec![];
    let mut saturday = vec![];
    let mut sunday = vec![];

    if let Some(message) = message.get_read_property_multiple_ack_into() {
        for values in message {
            for x in values.property_results.into_iter() {
                match x.value {
                    PropertyValue::PropValue(ApplicationDataValue::WeeklySchedule(
                        weekly_schedule,
                    )) => {
                        monday = weekly_schedule.monday.collect();
                        tuesday = weekly_schedule.tuesday.collect();
                        wednesday = weekly_schedule.wednesday.collect();
                        thursday = weekly_schedule.thursday.collect();
                        friday = weekly_schedule.friday.collect();
                        saturday = weekly_schedule.saturday.collect();
                        sunday = weekly_schedule.sunday.collect();
                    }
                    _ => {
                        // do nothing
                    }
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
    socket.send_to(buf, IP_ADDRESS)?;
    println!("Sent:     {:02x?} to {}\n", buf, IP_ADDRESS);

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
