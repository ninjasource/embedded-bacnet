// cargo run --example learn_controller_no_alloc --no-default-features -- --addr "192.168.1.249:47808" --device-id 79079

#![allow(unused_imports)]

use std::collections::HashMap;

use clap::{command, Parser};
use common::MySocket;
use embedded_bacnet::{
    application_protocol::{
        primitives::data_value::{ApplicationDataValue, BitString, Enumerated},
        services::{
            read_property::{ReadProperty, ReadPropertyValue},
            read_property_multiple::{
                PropertyValue, ReadPropertyMultiple, ReadPropertyMultipleObject,
            },
        },
    },
    common::{
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
        spec::{Binary, EngineeringUnits, Status},
        time_value::TimeValue,
    },
    simple::{Bacnet, BacnetError},
};

mod common;

#[cfg(feature = "alloc")]
fn main() {}

/// A Bacnet Client example to discover the capabilities of a controller
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address with port e.g. "192.168.1.249:47808"
    #[arg(short, long)]
    addr: String,

    /// Device ID of the controller e.g. 79079
    #[arg(short, long)]
    device_id: u32,
}

#[cfg(not(feature = "alloc"))]
#[tokio::main]
async fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    let args = Args::parse();
    let mut bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 1500];

    // fetch object list
    let object_id = ObjectId::new(ObjectType::ObjectDevice, args.device_id);
    let request = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let result = bacnet.read_property(&mut buf, request).await?;

    let mut map = HashMap::new();
    if let ReadPropertyValue::ObjectIdList(list) = result.property_value {
        // put all objects in their respective bins by object type
        for item in list.into_iter() {
            let item = item?;
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
        let object_type = ObjectType::try_from(*object_type).unwrap();
        match object_type {
            ObjectType::ObjectBinaryInput
            | ObjectType::ObjectBinaryOutput
            | ObjectType::ObjectBinaryValue => {
                for chunk in ids.as_slice().chunks(10).into_iter() {
                    let _values = get_multi_binary(&mut bacnet, &mut buf, chunk).await?;
                    println!("{:?}", _values);
                }
            }
            ObjectType::ObjectAnalogInput
            | ObjectType::ObjectAnalogOutput
            | ObjectType::ObjectAnalogValue => {
                for chunk in ids.as_slice().chunks(10).into_iter() {
                    let _values = get_multi_analog(&mut bacnet, &mut buf, chunk).await?;
                    println!("{:?}", _values);
                }
            }
            ObjectType::ObjectSchedule => {
                for object_id in ids.as_slice() {
                    let values = get_multi_schedule(&mut bacnet, &mut buf, object_id).await?;
                    println!("{:?}", values);
                }
            }
            ObjectType::ObjectTrendlog => {
                for chunk in ids.as_slice().chunks(10).into_iter() {
                    let values = get_multi_trend_log(&mut bacnet, &mut buf, chunk).await?;
                    println!("{:?}", values);
                }
            }

            _ => {}
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
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct BinaryValue {
    pub id: ObjectId,
    pub name: String,
    pub value: bool,
    pub status: Status,
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

#[cfg(not(feature = "alloc"))]
async fn get_multi_binary(
    bacnet: &mut Bacnet<MySocket>,
    buf: &mut [u8],
    object_ids: &[ObjectId],
) -> Result<Vec<BinaryValue>, BacnetError<MySocket>> {
    let property_ids = [
        PropertyId::PropObjectName,
        PropertyId::PropPresentValue,
        PropertyId::PropStatusFlags,
    ];
    let items: Vec<ReadPropertyMultipleObject> = object_ids
        .iter()
        .map(|x| ReadPropertyMultipleObject::new(x.clone(), &property_ids))
        .collect();
    let request = ReadPropertyMultiple::new(&items);
    let result = bacnet.read_property_multiple(buf, request).await?;

    let mut items = vec![];
    for obj in &result {
        let obj = obj?;
        let mut x = obj.property_results.into_iter();
        let name = x.next().unwrap()?.value.to_string();
        let value = match x.next().unwrap()?.value {
            PropertyValue::PropValue(ApplicationDataValue::Enumerated(Enumerated::Binary(
                Binary::On,
            ))) => true,
            _ => false,
        };
        let status = match x.next().unwrap()?.value {
            PropertyValue::PropValue(ApplicationDataValue::BitString(BitString::Status(x))) => x,
            _ => unreachable!(),
        };

        items.push(BinaryValue {
            id: obj.object_id,
            name,
            value,
            status,
        });
    }

    return Ok(items);
}

#[cfg(not(feature = "alloc"))]
async fn get_multi_analog(
    bacnet: &mut Bacnet<MySocket>,
    buf: &mut [u8],
    object_ids: &[ObjectId],
) -> Result<Vec<AnalogValue>, BacnetError<MySocket>> {
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

    let request = ReadPropertyMultiple::new(&items);
    let result = bacnet.read_property_multiple(buf, request).await?;

    let mut items = vec![];
    for obj in &result {
        let obj = obj?;
        let mut x = obj.property_results.into_iter();
        let name = x.next().unwrap()?.value.to_string();
        let value = match x.next().unwrap()?.value {
            PropertyValue::PropValue(ApplicationDataValue::Real(val)) => val,
            _ => unreachable!(),
        };
        let units = match x.next().unwrap()?.value {
            PropertyValue::PropValue(ApplicationDataValue::Enumerated(Enumerated::Units(u))) => {
                u.clone()
            }
            _ => unreachable!(),
        };
        let status = match x.next().unwrap()?.value {
            PropertyValue::PropValue(ApplicationDataValue::BitString(BitString::Status(x))) => x,
            _ => unreachable!(),
        };

        items.push(AnalogValue {
            id: obj.object_id,
            name,
            value,
            units,
            status,
        })
    }

    return Ok(items);
}

#[cfg(not(feature = "alloc"))]
async fn get_multi_trend_log(
    bacnet: &mut Bacnet<MySocket>,
    buf: &mut [u8],
    object_ids: &[ObjectId],
) -> Result<Vec<TrendLogValue>, BacnetError<MySocket>> {
    let property_ids = [PropertyId::PropObjectName, PropertyId::PropRecordCount];

    let items: Vec<ReadPropertyMultipleObject> = object_ids
        .iter()
        .map(|x| ReadPropertyMultipleObject::new(x.clone(), &property_ids))
        .collect();

    let request = ReadPropertyMultiple::new(&items);
    let result = bacnet.read_property_multiple(buf, request).await?;

    let mut items = vec![];

    for obj in &result {
        let obj = obj?;
        let mut x = obj.property_results.into_iter();
        let name = x.next().unwrap()?.value.to_string();
        let record_count = match x.next().unwrap()?.value {
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

#[cfg(not(feature = "alloc"))]
async fn get_multi_schedule(
    bacnet: &mut Bacnet<MySocket>,
    buf: &mut [u8],
    object_id: &ObjectId,
) -> Result<Vec<ScheduleValue>, BacnetError<MySocket>> {
    let property_ids = [PropertyId::PropObjectName, PropertyId::PropWeeklySchedule];
    let objects = [ReadPropertyMultipleObject::new(
        object_id.clone(),
        &property_ids,
    )];
    let request = ReadPropertyMultiple::new(&objects);
    let result = bacnet.read_property_multiple(buf, request).await?;

    let mut items = vec![];

    for obj in &result {
        let obj = obj?;
        let mut x = obj.property_results.into_iter();
        let name = x.next().unwrap()?.value.to_string();
        let value = match x.next().unwrap()?.value {
            PropertyValue::PropValue(ApplicationDataValue::WeeklySchedule(schedule)) => schedule,
            _ => panic!("expected weekly schedule"),
        };

        items.push(ScheduleValue {
            id: obj.object_id,
            name,
            monday: value.monday.into_iter().map(|x| x.unwrap()).collect(),
            tuesday: value.tuesday.into_iter().map(|x| x.unwrap()).collect(),
            wednesday: value.wednesday.into_iter().map(|x| x.unwrap()).collect(),
            thursday: value.thursday.into_iter().map(|x| x.unwrap()).collect(),
            friday: value.friday.into_iter().map(|x| x.unwrap()).collect(),
            saturday: value.saturday.into_iter().map(|x| x.unwrap()).collect(),
            sunday: value.sunday.into_iter().map(|x| x.unwrap()).collect(),
        });
    }

    return Ok(items);
}
