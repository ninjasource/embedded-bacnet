//#![allow(dead_code, unreachable_code, unused_variables)]
#![allow(dead_code)]

use std::{collections::HashMap, io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::{ConfirmedRequest, ConfirmedRequestSerivice},
        primitives::data_value::{ApplicationDataValue, BitString, Enumerated},
        read_property::ReadProperty,
        read_property_multiple::{PropertyValue, ReadPropertyMultiple, ReadPropertyMultipleObject},
    },
    common::{
        helper::{Buffer, Reader},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
        spec::{Binary, EngineeringUnits, StatusFlags},
    },
    network_protocol::data_link::DataLink,
};
use flagset::FlagSet;

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectDevice, 79079);
    let read_property = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadProperty(read_property));
    let data_link = DataLink::new_confirmed_req(req);
    let mut buffer = Buffer::new();
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    // receive reply
    let mut buf = vec![0; 64 * 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::new(buf.len());
    let message = DataLink::decode(&mut reader, buf).unwrap();

    if let Some(ack) = message.get_read_property_ack() {
        let mut map = HashMap::new();

        // put all object in their respective bins by object type
        for item in &ack.properties {
            match item.object_type {
                ObjectType::ObjectBinaryOutput
                | ObjectType::ObjectBinaryInput
                | ObjectType::ObjectBinaryValue
                | ObjectType::ObjectAnalogInput
                | ObjectType::ObjectAnalogOutput
                | ObjectType::ObjectAnalogValue => {
                    let list = map.entry(item.object_type as u32).or_insert(vec![]);
                    list.push(item);
                }
                _ => {}
            }
        }

        for (object_type, ids) in map.iter() {
            let object_type = ObjectType::from(*object_type);
            match object_type {
                ObjectType::ObjectBinaryInput
                | ObjectType::ObjectBinaryOutput
                | ObjectType::ObjectBinaryValue => {
                    for chunk in ids.as_slice().chunks(10).into_iter() {
                        let _values = get_multi_binary(&socket, chunk)?;
                        println!("{:?}", _values);
                    }
                }
                ObjectType::ObjectAnalogInput
                | ObjectType::ObjectAnalogOutput
                | ObjectType::ObjectAnalogValue => {
                    for chunk in ids.as_slice().chunks(10).into_iter() {
                        let _values = get_multi_analog(&socket, chunk)?;
                        println!("{:?}", _values);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct AnalogValue {
    pub id: ObjectId,
    pub name: String,
    pub value: f32,
    pub units: EngineeringUnits,
    pub status_flags: FlagSet<StatusFlags>,
}

#[derive(Debug)]
struct BinaryValue {
    pub id: ObjectId,
    pub name: String,
    pub value: bool,
    pub status_flags: FlagSet<StatusFlags>,
}

fn get_multi_binary(
    socket: &UdpSocket,
    object_ids: &[&ObjectId],
) -> Result<Vec<BinaryValue>, Error> {
    let items = object_ids
        .iter()
        .map(|x| {
            let mut property_ids = Vec::new();
            property_ids.push(PropertyId::PropObjectName);
            property_ids.push(PropertyId::PropPresentValue);
            property_ids.push(PropertyId::PropStatusFlags);
            ReadPropertyMultipleObject::new(**x, property_ids)
        })
        .collect();

    let rpm = ReadPropertyMultiple::new(items);
    let buffer = read_property_multiple_to_bytes(rpm);
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buffer.to_bytes(), &addr)?;
    let mut buf = vec![0; 16 * 1024];
    let (n, _) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    let mut reader = Reader::new(buf.len());
    let message = DataLink::decode(&mut reader, buf).unwrap();

    //let message = send_and_recv(items, socket)?;

    if let Some(ack) = message.get_read_property_multiple_ack() {
        let mut items = vec![];

        while let Some(x) = ack.decode_next(&mut reader, buf) {
            let name = x.decode_next(&mut reader, buf).unwrap().value.to_string();
            let value = match x.decode_next(&mut reader, buf).unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::Enumerated(Enumerated::Binary(
                    Binary::On,
                ))) => true,
                _ => false,
            };
            let status_flags = match x.decode_next(&mut reader, buf).unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::BitString(
                    BitString::StatusFlags(x),
                )) => x,
                _ => unreachable!(),
            };

            assert!(x.decode_next(&mut reader, buf).is_none());

            items.push(BinaryValue {
                id: x.object_id,
                name,
                value,
                status_flags,
            });
        }

        return Ok(items);
    }

    Ok(vec![])
}

fn get_multi_analog(
    socket: &UdpSocket,
    object_ids: &[&ObjectId],
) -> Result<Vec<AnalogValue>, Error> {
    let items = object_ids
        .iter()
        .map(|x| {
            let mut property_ids = Vec::new();
            property_ids.push(PropertyId::PropObjectName);
            property_ids.push(PropertyId::PropPresentValue);
            property_ids.push(PropertyId::PropUnits);
            property_ids.push(PropertyId::PropStatusFlags);
            ReadPropertyMultipleObject::new(**x, property_ids)
        })
        .collect();

    let rpm = ReadPropertyMultiple::new(items);
    let buffer = read_property_multiple_to_bytes(rpm);
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buffer.to_bytes(), &addr)?;
    let mut buf = vec![0; 16 * 1024];
    let (n, _) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    let mut reader = Reader::new(buf.len());
    let message = DataLink::decode(&mut reader, buf).unwrap();

    if let Some(ack) = message.get_read_property_multiple_ack() {
        let mut items = vec![];

        while let Some(x) = ack.decode_next(&mut reader, buf) {
            let name = x.decode_next(&mut reader, buf).unwrap().value.to_string();
            let value = match x.decode_next(&mut reader, buf).unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::Real(val)) => val,
                _ => unreachable!(),
            };
            let units = match x.decode_next(&mut reader, buf).unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::Enumerated(Enumerated::Units(
                    u,
                ))) => u.clone(),
                _ => unreachable!(),
            };
            let status_flags = match x.decode_next(&mut reader, buf).unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::BitString(
                    BitString::StatusFlags(x),
                )) => x,
                _ => unreachable!(),
            };

            assert!(x.decode_next(&mut reader, buf).is_none());

            items.push(AnalogValue {
                id: x.object_id,
                name,
                value,
                units,
                status_flags,
            })
        }

        return Ok(items);
    }

    Ok(vec![])
}

fn send_and_recv(
    items: Vec<ReadPropertyMultipleObject>,
    socket: &UdpSocket,
) -> Result<DataLink, Error> {
    let rpm = ReadPropertyMultiple::new(items);
    let buffer = read_property_multiple_to_bytes(rpm);
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buffer.to_bytes(), &addr)?;
    let mut buf = vec![0; 16 * 1024];
    let (n, _) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    let mut reader = Reader::new(buf.len());
    let message = DataLink::decode(&mut reader, buf).unwrap();
    Ok(message)
}

fn read_property_multiple_to_bytes(rpm: ReadPropertyMultiple) -> Buffer {
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadPropertyMultiple(rpm));
    let data_link = DataLink::new_confirmed_req(req);
    let mut buffer = Buffer::new();
    data_link.encode(&mut buffer);
    buffer
}
