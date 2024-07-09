// cargo run --example update_schedule -- --addr "192.168.1.249:47808"

#![allow(unused_imports)]

use clap::{command, Parser};
use common::MySocket;
use embedded_bacnet::{
    application_protocol::{
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
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    simple::{Bacnet, BacnetError},
};

mod common;

/// A Bacnet Client example to update a schedule
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address with port e.g. "192.168.1.249:47808"
    #[arg(short, long)]
    addr: String,
}

#[cfg(not(feature = "alloc"))]
fn main() {}

#[cfg(feature = "alloc")]
#[tokio::main]
async fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    let args = Args::parse();
    let mut bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 1500];

    // get schedule
    let mut weekly_schedule = decode_weekly_schedule(&mut bacnet, &mut buf).await?;
    println!("Monday: {:?}", weekly_schedule.monday);

    // change the schedule
    weekly_schedule.monday[0].time.hour = 9;
    println!("{:?}", weekly_schedule);

    let request = WriteProperty::new(
        ObjectId::new(ObjectType::ObjectSchedule, 1),
        PropertyId::PropWeeklySchedule,
        None,
        None,
        ApplicationDataValueWrite::WeeklySchedule(weekly_schedule),
    );

    let ack = bacnet.write_property(&mut buf, request).await?;
    println!("Write ack: {:?}", ack);

    Ok(())
}

#[cfg(feature = "alloc")]
async fn decode_weekly_schedule<'a>(
    bacnet: &mut Bacnet<MySocket>,
    buf: &mut [u8],
) -> Result<WeeklySchedule<'a>, BacnetError<MySocket>> {
    // fetch
    let rpm = ReadPropertyMultipleObject::new(
        ObjectId::new(ObjectType::ObjectSchedule, 1),
        vec![PropertyId::PropObjectName, PropertyId::PropWeeklySchedule],
    );
    let request = ReadPropertyMultiple::new(vec![rpm]);
    let result = bacnet.read_property_multiple(buf, request).await?;

    for values in result.objects_with_results {
        for x in values.property_results {
            match x.value {
                PropertyValue::PropValue(ApplicationDataValue::WeeklySchedule(weekly_schedule)) => {
                    return Ok(weekly_schedule)
                }
                _ => {
                    // do nothing
                }
            }
        }
    }

    panic!("schedule not found")
}
