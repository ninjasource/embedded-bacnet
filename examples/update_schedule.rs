// cargo run --example update_schedule -- --addr "192.168.1.249:47808"

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
    simple::BacnetError,
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

#[tokio::main]
async fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    let args = Args::parse();
    let mut bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 4096];

    // fetch
    let object_id = ObjectId::new(ObjectType::ObjectSchedule, 1);
    let property_ids = [PropertyId::PropObjectName, PropertyId::PropWeeklySchedule];
    let rpm = ReadPropertyMultipleObject::new(object_id, &property_ids);
    let objects = [rpm];
    let request = ReadPropertyMultiple::new(&objects);
    let result = bacnet.read_property_multiple(&mut buf, request).await?;

    let mut monday = vec![];
    let mut tuesday = vec![];
    let mut wednesday = vec![];
    let mut thursday = vec![];
    let mut friday = vec![];
    let mut saturday = vec![];
    let mut sunday = vec![];

    for values in &result {
        let values = values?;
        for x in values.property_results.into_iter() {
            let x = x?;
            match x.value {
                PropertyValue::PropValue(ApplicationDataValue::WeeklySchedule(weekly_schedule)) => {
                    monday = weekly_schedule
                        .monday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    tuesday = weekly_schedule
                        .tuesday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    wednesday = weekly_schedule
                        .wednesday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    thursday = weekly_schedule
                        .thursday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    friday = weekly_schedule
                        .friday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    saturday = weekly_schedule
                        .saturday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                    sunday = weekly_schedule
                        .sunday
                        .into_iter()
                        .map(|x| x.unwrap())
                        .collect();
                }
                _ => {
                    // do nothing
                }
            }
        }
    }

    println!("Monday: {:?}", monday);

    // change the schedule
    monday[0].time.hour = 9;

    let weekly_schedule = WeeklySchedule::new(
        &monday, &tuesday, &wednesday, &thursday, &friday, &saturday, &sunday,
    );

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
