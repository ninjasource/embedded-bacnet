use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ComplexAckService, ConfirmedRequest, ConfirmedRequestSerivice},
        primitives::data_value::ApplicationDataValue,
        services::{
            read_property::{ReadProperty, ReadPropertyValue},
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

//const IP_ADDRESS: &str = "192.168.1.215:47808";
const IP_ADDRESS: &str = "192.168.1.249:47808";

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;
    let object_id = ObjectId::new(ObjectType::ObjectTrendlog, 4);

    let record_count = get_record_count(&socket, object_id)?;
    println!("Record count {record_count}");

    const MAX_COUNT_PER_REQ: usize = 55;
    for index in (1..=record_count).step_by(MAX_COUNT_PER_REQ) {
        print_items_by_range(&socket, object_id, index, MAX_COUNT_PER_REQ)?;
    }

    Ok(())
}

fn print_items_by_range(
    socket: &UdpSocket,
    object_id: ObjectId,
    index: u32,
    count: usize,
) -> Result<(), Error> {
    // encode packet
    let request_type = ReadRangeRequestType::ByPosition(ReadRangeByPosition {
        index: index,
        count: count as u32,
    });
    let rp = ReadRange::new(object_id, PropertyId::PropLogBuffer, request_type);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadRange(rp));
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
    //println!("Sent:     {:02x?} to {}\n", buf, IP_ADDRESS);

    // receive reply
    let mut buf = vec![0; 4096];
    let (n, _peer) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    //println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::new();
    let message = DataLink::decode(&mut reader, buf).unwrap();
    //println!("Decoded:  {:?}\n", message);

    if let Some(ack) = message.get_ack_into() {
        match ack.service {
            ComplexAckService::ReadRange(read_range) => {
                for item in read_range.item_data {
                    let value = match item.value {
                        ReadRangeValue::Real(x) => x,
                        _ => 0.0,
                    };
                    println!(
                        "{}-{:02}-{:02} {:02}:{:02} - {:.2}",
                        item.date.year,
                        item.date.month,
                        item.date.day,
                        item.time.hour,
                        item.time.minute,
                        value
                    );
                }
            }
            _ => {
                // do nothing
            }
        }
    }

    Ok(())
}

fn get_record_count(socket: &UdpSocket, object_id: ObjectId) -> Result<u32, Error> {
    // encode packet
    let rp = ReadProperty::new(object_id, PropertyId::PropRecordCount);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadProperty(rp));
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
    if let Some(message) = message.get_read_property_ack_into() {
        if let ReadPropertyValue::ApplicationDataValue(ApplicationDataValue::UnsignedInt(x)) =
            message.property_value
        {
            Ok(x)
        } else {
            Ok(0)
        }
    } else {
        Ok(0)
    }
}
