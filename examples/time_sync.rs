// cargo run --example time_sync -- --addr "192.168.1.249:47808" --device-id 79079

#![allow(unused_imports)]
use chrono::{Datelike, Local, Timelike};
use clap::{command, Parser};
use common::MySocket;
use embedded_bacnet::{
    application_protocol::{
        primitives::data_value::{Date, Time},
        services::{
            read_property_multiple::{ReadPropertyMultiple, ReadPropertyMultipleObject},
            time_synchronization::TimeSynchronization,
        },
    },
    common::{
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    simple::{Bacnet, BacnetError},
};

mod common;

/// A Bacnet Client example set the time on the controller to the system time (on this pc)
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
fn main() {}

#[cfg(feature = "alloc")]
#[tokio::main]
async fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    let args = Args::parse();
    let mut bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 1500];

    set_time_to_now(&mut bacnet, &mut buf).await?;
    request_date_time(args.device_id, &mut bacnet, &mut buf).await?;

    Ok(())
}

#[cfg(feature = "alloc")]
async fn set_time_to_now(
    bacnet: &mut Bacnet<MySocket>,
    buf: &mut [u8],
) -> Result<(), BacnetError<MySocket>> {
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
    let request = TimeSynchronization { date, time };
    bacnet.time_sync(buf, request).await?;
    println!("Controller date time set to {:?}", now);
    Ok(())
}

#[cfg(feature = "alloc")]
async fn request_date_time(
    device_id: u32,
    bacnet: &mut Bacnet<MySocket>,
    buf: &mut [u8],
) -> Result<(), BacnetError<MySocket>> {
    println!("Fetching date time from controller:");

    let rpm = ReadPropertyMultipleObject::new(
        ObjectId::new(ObjectType::ObjectDevice, device_id),
        vec![PropertyId::PropLocalDate, PropertyId::PropLocalTime],
    );
    let request = ReadPropertyMultiple::new(vec![rpm]);
    let result = bacnet.read_property_multiple(buf, request).await?;

    // read values
    for values in result.objects_with_results {
        for x in values.property_results {
            println!("{:?}", x);
        }
    }

    Ok(())
}
