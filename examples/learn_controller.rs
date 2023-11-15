use std::{collections::HashMap, io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        confirmed::{ConfirmedRequest, ConfirmedRequestService},
        primitives::data_value::{ApplicationDataValue, BitString, Enumerated},
        services::{
            read_property::{ReadProperty, ReadPropertyValue},
            read_property_multiple::{
                PropertyValue, ReadPropertyMultiple, ReadPropertyMultipleObject,
            },
        },
    },
    common::{
        io::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
        spec::{Binary, EngineeringUnits, StatusFlags},
        time_value::TimeValue,
    },
    network_protocol::data_link::DataLink,
};
use flagset::FlagSet;

const IP_ADDRESS: &str = "192.168.1.249:47808";
const DEVICE_ID: u32 = 79079;

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC1))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectDevice, DEVICE_ID);
    let read_property = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::ReadProperty(read_property));
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
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf).unwrap();
    println!("Decoded: {:?}", message);

    if let Some(ack) = message.get_read_property_ack_into() {
        let mut map = HashMap::new();

        if let ReadPropertyValue::ObjectIdList(list) = ack.property_value {
            // put all objects in their respective bins by object type
            for item in list.into_iter() {
                let item = item.unwrap();
                match item.object_type {
                    ObjectType::ObjectBinaryOutput
                    | ObjectType::ObjectBinaryInput
                    | ObjectType::ObjectBinaryValue
                    | ObjectType::ObjectAnalogInput
                    | ObjectType::ObjectAnalogOutput
                    | ObjectType::ObjectAnalogValue
                    | ObjectType::ObjectSchedule
                    | ObjectType::ObjectTrendlog => {
                        let list = map.entry(item.object_type.clone() as u32).or_insert(vec![]);
                        list.push(item);
                    }
                    _ => {}
                }
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
                ObjectType::ObjectSchedule => {
                    for object_id in ids.as_slice() {
                        let values = get_multi_schedule(&socket, object_id)?;
                        println!("{:?}", values);
                    }
                }
                ObjectType::ObjectTrendlog => {
                    for chunk in ids.as_slice().chunks(10).into_iter() {
                        let values = get_multi_trend_log(&socket, chunk)?;
                        println!("{:?}", values);
                    }
                }

                _ => {}
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct AnalogValue {
    pub id: ObjectId,
    pub name: String,
    pub value: f32,
    pub units: EngineeringUnits,
    pub status_flags: FlagSet<StatusFlags>,
}

#[derive(Debug, Clone)]
pub struct BinaryValue {
    pub id: ObjectId,
    pub name: String,
    pub value: bool,
    pub status_flags: FlagSet<StatusFlags>,
}

#[derive(Debug, Clone)]
pub struct ScheduleValue {
    pub id: ObjectId,
    pub name: String,
    pub monday: Vec<TimeValue>,
    pub tuesday: Vec<TimeValue>,
    pub wednesday: Vec<TimeValue>,
    pub thursday: Vec<TimeValue>,
    pub friday: Vec<TimeValue>,
    pub saturday: Vec<TimeValue>,
    pub sunday: Vec<TimeValue>,
}

#[derive(Debug, Clone)]
pub struct TrendLogValue {
    pub id: ObjectId,
    pub name: String,
    pub record_count: u32,
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
        .map(|x| ReadPropertyMultipleObject::new(x.clone(), &property_ids))
        .collect();
    let rpm = ReadPropertyMultiple::new(&items);
    let mut buf = vec![0; 16 * 1024];
    let mut writer = Writer::new(&mut buf);
    read_property_multiple_to_bytes(rpm, &mut writer);
    socket.send_to(writer.to_bytes(), &IP_ADDRESS)?;
    let mut buf = vec![0; 16 * 1024];
    let (n, _) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf).unwrap();

    if let Some(ack) = message.get_read_property_multiple_ack_into() {
        let mut items = vec![];

        for obj in ack {
            let mut x = obj.property_results.into_iter();
            let name = x.next().unwrap().unwrap().value.to_string();
            let value = match x.next().unwrap().unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::Enumerated(Enumerated::Binary(
                    Binary::On,
                ))) => true,
                _ => false,
            };
            let status_flags = match x.next().unwrap().unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::BitString(
                    BitString::StatusFlags(x),
                )) => x,
                _ => unreachable!(),
            };

            items.push(BinaryValue {
                id: obj.object_id,
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
        .map(|x| ReadPropertyMultipleObject::new(x.clone(), &property_ids))
        .collect();

    let rpm = ReadPropertyMultiple::new(&items);
    let mut buf = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buf);
    read_property_multiple_to_bytes(rpm, &mut buffer);
    socket.send_to(buffer.to_bytes(), &IP_ADDRESS)?;
    let mut buf = vec![0; 16 * 1024];
    let (n, _) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf).unwrap();

    if let Some(ack) = message.get_read_property_multiple_ack_into() {
        let mut items = vec![];

        for obj in ack {
            let mut x = obj.property_results.into_iter();
            let name = x.next().unwrap().unwrap().value.to_string();
            let value = match x.next().unwrap().unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::Real(val)) => val,
                _ => unreachable!(),
            };
            let units = match x.next().unwrap().unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::Enumerated(Enumerated::Units(
                    u,
                ))) => u.clone(),
                _ => unreachable!(),
            };
            let status_flags = match x.next().unwrap().unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::BitString(
                    BitString::StatusFlags(x),
                )) => x,
                _ => FlagSet::default(), // ignore property read errors
            };

            items.push(AnalogValue {
                id: obj.object_id,
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

fn get_multi_trend_log(
    socket: &UdpSocket,
    object_ids: &[ObjectId],
) -> Result<Vec<TrendLogValue>, Error> {
    let property_ids = [PropertyId::PropObjectName, PropertyId::PropRecordCount];

    let items: Vec<ReadPropertyMultipleObject> = object_ids
        .iter()
        .map(|x| ReadPropertyMultipleObject::new(x.clone(), &property_ids))
        .collect();

    let rpm = ReadPropertyMultiple::new(&items);
    let mut buf = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buf);
    read_property_multiple_to_bytes(rpm, &mut buffer);
    socket.send_to(buffer.to_bytes(), &IP_ADDRESS)?;
    let mut buf = vec![0; 16 * 1024];
    let (n, _) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf).unwrap();

    if let Some(ack) = message.get_read_property_multiple_ack_into() {
        let mut items = vec![];

        for obj in ack {
            let mut x = obj.property_results.into_iter();
            let name = x.next().unwrap().unwrap().value.to_string();
            let record_count = match x.next().unwrap().unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::UnsignedInt(val)) => val,
                _ => unreachable!(),
            };

            items.push(TrendLogValue {
                id: obj.object_id,
                name,
                record_count,
            })
        }

        return Ok(items);
    }

    Ok(vec![])
}

fn get_multi_schedule(
    socket: &UdpSocket,
    object_id: &ObjectId,
) -> Result<Vec<ScheduleValue>, Error> {
    let property_ids = [PropertyId::PropObjectName, PropertyId::PropWeeklySchedule];
    let objects = [ReadPropertyMultipleObject::new(
        object_id.clone(),
        &property_ids,
    )];
    let rpm = ReadPropertyMultiple::new(&objects);
    let mut buf = vec![0; 4 * 1024];
    let mut writer = Writer::new(&mut buf);
    read_property_multiple_to_bytes(rpm, &mut writer);
    socket.send_to(writer.to_bytes(), &IP_ADDRESS)?;
    let mut buf = vec![0; 16 * 1024];
    let (n, _) = socket.recv_from(&mut buf).unwrap();
    let buf = &buf[..n];
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf).unwrap();

    if let Some(ack) = message.get_read_property_multiple_ack_into() {
        let mut items = vec![];

        for obj in ack {
            let mut x = obj.property_results.into_iter();
            let name = x.next().unwrap().unwrap().value.to_string();
            let value = match x.next().unwrap().unwrap().value {
                PropertyValue::PropValue(ApplicationDataValue::WeeklySchedule(schedule)) => {
                    schedule
                }
                _ => panic!("expected weekly schedule"),
            };

            items.push(ScheduleValue {
                id: obj.object_id,
                name,
                monday: value.monday.map(|x| x.unwrap()).collect(),
                tuesday: value.tuesday.map(|x| x.unwrap()).collect(),
                wednesday: value.wednesday.map(|x| x.unwrap()).collect(),
                thursday: value.thursday.map(|x| x.unwrap()).collect(),
                friday: value.friday.map(|x| x.unwrap()).collect(),
                saturday: value.saturday.map(|x| x.unwrap()).collect(),
                sunday: value.sunday.map(|x| x.unwrap()).collect(),
            });
        }

        return Ok(items);
    }

    Ok(vec![])
}

fn read_property_multiple_to_bytes(rpm: ReadPropertyMultiple, writer: &mut Writer) {
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::ReadPropertyMultiple(rpm));
    let data_link = DataLink::new_confirmed_req(req);
    data_link.encode(writer);
}
