// cargo run --example read_property_multiple_all -- --addr "192.168.1.249:47808"
// cargo run --example read_property_multiple_all --no-default-features -- --addr "192.168.1.249:47808"

use clap::{command, Parser};
use common::MySocket;
use embedded_bacnet::{
    application_protocol::services::read_property_multiple::{
        ReadPropertyMultiple, ReadPropertyMultipleObject,
    },
    common::{
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    simple::BacnetError,
};

mod common;

/// A Bacnet Client example to read all the property values for analog input #1
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address with port e.g. "192.168.1.249:47808"
    #[arg(short, long)]
    addr: String,
}

#[cfg(feature = "alloc")]
#[tokio::main]
async fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    simple_logger::init().unwrap();
    let args = Args::parse();
    let mut bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 1500];

    // fetch all property values for an analog input 4
    let objects = vec![ReadPropertyMultipleObject::new(
        ObjectId::new(ObjectType::ObjectAnalogInput, 1),
        vec![PropertyId::PropAll],
    )];
    let request = ReadPropertyMultiple::new(objects);
    let result = bacnet.read_property_multiple(&mut buf, request).await?;

    // print
    for values in &result.objects_with_results {
        for x in &values.property_results {
            println!("{:?}", x);
        }
    }

    Ok(())
}

#[cfg(not(feature = "alloc"))]
#[tokio::main]
async fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    simple_logger::init().unwrap();
    let args = Args::parse();
    let mut bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 1500];

    // fetch all property values for an analog input 4
    let objects = [ReadPropertyMultipleObject::new(
        ObjectId::new(ObjectType::ObjectAnalogInput, 1),
        &[PropertyId::PropAll],
    )];
    let request = ReadPropertyMultiple::new(&objects);
    let result = bacnet.read_property_multiple(&mut buf, request).await?;

    // print
    for values in &result {
        for x in &values?.property_results {
            println!("{:?}", x?);
        }
    }

    Ok(())
}
