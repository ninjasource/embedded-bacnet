// A Bacnet Client example to read a range of values from trend log #4 (typically used for displaying a chart)
// cargo run --example read_range -- --addr "192.168.1.249:47808"
// cargo run --example read_range --no-default-features -- --addr "192.168.1.249:47808"

#![allow(unused_imports)]

use core::ops::Range;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use clap::{command, Parser};
use common::MySocket;
use embedded_bacnet::{
    application_protocol::{
        primitives::data_value::ApplicationDataValue,
        services::{
            read_property::{ReadProperty, ReadPropertyValue},
            read_range::{ReadRange, ReadRangeByPosition, ReadRangeRequestType, ReadRangeValue},
        },
    },
    common::{
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    simple::{Bacnet, BacnetError},
};

mod common;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address with port e.g. "192.168.1.249:47808"
    #[arg(short, long)]
    addr: String,
}

#[tokio::main]
async fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    let args = Args::parse();
    let mut bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 1500];

    // fetch record count
    let object_id = ObjectId::new(ObjectType::ObjectTrendlog, 1);
    let record_count = get_record_count(&mut bacnet, &mut buf, object_id.clone()).await?;
    println!("Record count {record_count}");

    // fetch records in batches and print
    const MAX_LOG_COUNT_PER_REQ: usize = 55;
    for row in (1..=record_count).step_by(MAX_LOG_COUNT_PER_REQ) {
        get_items_for_range(
            &mut bacnet,
            &mut buf,
            object_id.clone(),
            row..MAX_LOG_COUNT_PER_REQ,
        )
        .await?;
    }

    Ok(())
}

async fn get_record_count(
    bacnet: &mut Bacnet<MySocket>,
    buf: &mut [u8],
    object_id: ObjectId,
) -> Result<usize, BacnetError<MySocket>> {
    let request = ReadProperty::new(object_id, PropertyId::PropRecordCount);
    let result = bacnet.read_property(buf, request).await?;

    if let ReadPropertyValue::ApplicationDataValue(ApplicationDataValue::UnsignedInt(x)) =
        result.property_value
    {
        Ok(x as usize)
    } else {
        Ok(0)
    }
}

#[cfg(not(feature = "alloc"))]
async fn get_items_for_range(
    bacnet: &mut Bacnet<MySocket>,
    buf: &mut [u8],
    object_id: ObjectId,
    range: Range<usize>,
) -> Result<(), BacnetError<MySocket>> {
    let request_type = ReadRangeRequestType::ByPosition(ReadRangeByPosition {
        index: range.start as u32,
        count: range.end as u32,
    });
    let request = ReadRange::new(object_id, PropertyId::PropLogBuffer, request_type);
    let result = bacnet.read_range(buf, request).await?;

    for item in &result.item_data {
        let item = item?;
        let value = match item.value {
            ReadRangeValue::Real(x) => x,
            _ => 0.0,
        };
        let date_time = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(
                item.date.year as i32,
                item.date.month as u32,
                item.date.day as u32,
            )
            .unwrap(),
            NaiveTime::from_hms_opt(item.time.hour as u32, item.time.minute as u32, 0).unwrap(),
        );

        println!("{} {}", date_time, value);
    }

    Ok(())
}

#[cfg(feature = "alloc")]
async fn get_items_for_range(
    bacnet: &mut Bacnet<MySocket>,
    buf: &mut [u8],
    object_id: ObjectId,
    range: Range<usize>,
) -> Result<(), BacnetError<MySocket>> {
    let request_type = ReadRangeRequestType::ByPosition(ReadRangeByPosition {
        index: range.start as u32,
        count: range.end as u32,
    });
    let request = ReadRange::new(object_id, PropertyId::PropLogBuffer, request_type);
    let result = bacnet.read_range(buf, request).await?;

    for item in result.item_data.items {
        let value = match item.value {
            ReadRangeValue::Real(x) => x,
            _ => 0.0,
        };
        let date_time = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(
                item.date.year as i32,
                item.date.month as u32,
                item.date.day as u32,
            )
            .unwrap(),
            NaiveTime::from_hms_opt(item.time.hour as u32, item.time.minute as u32, 0).unwrap(),
        );

        println!("{} {}", date_time, value);
    }

    Ok(())
}
