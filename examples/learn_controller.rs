//#![allow(dead_code, unreachable_code, unused_variables)]
#![allow(dead_code)]

use std::{collections::HashMap, io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        confirmed::{ConfirmedRequest, ConfirmedRequestSerivice},
        primitives::data_value::{ApplicationDataValue, BitString, Enumerated},
        services::{
            read_property::{ReadProperty, ReadPropertyValue},
            read_property_multiple::{
                PropertyValue, ReadPropertyMultiple, ReadPropertyMultipleObject,
            },
        },
    },
    common::{
        helper::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
        spec::{Binary, EngineeringUnits, StatusFlags},
    },
    network_protocol::data_link::DataLink,
};
use flagset::FlagSet;

const IP_ADDRESS: &str = "192.168.1.215:47808";
const DEVICE_ID: u32 = 76011;

//const IP_ADDRESS: &str = "192.168.1.249:47808";
//const DEVICE_ID: u32 = 79079;

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectDevice, DEVICE_ID);
    let read_property = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadProperty(read_property));
    let data_link = DataLink::new_confirmed_req(req);
    let mut buf = vec![0; 16 * 1024];
    let mut writer = Writer::new(&mut buf);
    data_link.encode(&mut writer);

    // send packet
    let buf = writer.to_bytes();
    socket.send_to(buf, &IP_ADDRESS)?;
    println!("Sent:     {:02x?} to {}\n", buf, IP_ADDRESS);

    // receive reply
    let mut buf = vec![0; 64 * 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::new();
    let message = DataLink::decode(&mut reader, buf).unwrap();
    println!("Decoded: {:?}", message);

    if let Some(ack) = message.get_read_property_ack_into() {
        let mut map = HashMap::new();

        if let ReadPropertyValue::ObjectIdList(list) = ack.property_value {
            // put all objects in their respective bins by object type
            for item in list.into_iter() {
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

            /*
            // put all objects in their respective bins by object type
            while let Some(item) = list.decode_next(&mut reader, &buf) {
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
            }*/
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
    object_ids: &[ObjectId],
) -> Result<Vec<BinaryValue>, Error> {
    let property_ids = [
        PropertyId::PropObjectName,
        PropertyId::PropPresentValue,
        PropertyId::PropStatusFlags,
    ];

    let items: Vec<ReadPropertyMultipleObject> = object_ids
        .iter()
        .map(|x| ReadPropertyMultipleObject::new(*x, &property_ids))
        .collect();

    let rpm = ReadPropertyMultiple::new(&items);
    let mut buf = vec![0; 16 * 1024];
    let mut writer = Writer::new(&mut buf);
    read_property_multiple_to_bytes(rpm, &mut writer);
    socket.send_to(writer.to_bytes(), &IP_ADDRESS)?;
    let mut buf = vec![0; 16 * 1024];
    let (n, _) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    let mut reader = Reader::new();
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

            // you must do this
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
    object_ids: &[ObjectId],
) -> Result<Vec<AnalogValue>, Error> {
    let property_ids = [
        PropertyId::PropObjectName,
        PropertyId::PropPresentValue,
        PropertyId::PropUnits,
        PropertyId::PropStatusFlags,
    ];

    let items: Vec<ReadPropertyMultipleObject> = object_ids
        .iter()
        .map(|x| ReadPropertyMultipleObject::new(*x, &property_ids))
        .collect();

    let rpm = ReadPropertyMultiple::new(&items);
    let mut buf = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buf);
    read_property_multiple_to_bytes(rpm, &mut buffer);
    socket.send_to(buffer.to_bytes(), &IP_ADDRESS)?;
    let mut buf = vec![0; 16 * 1024];
    let (n, _) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    let mut reader = Reader::new();
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
                _ => FlagSet::default(), // ignore property read errors
            };

            // you must do this
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

fn read_property_multiple_to_bytes(rpm: ReadPropertyMultiple, writer: &mut Writer) {
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadPropertyMultiple(rpm));
    let data_link = DataLink::new_confirmed_req(req);
    data_link.encode(writer);
}
