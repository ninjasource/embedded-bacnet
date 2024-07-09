// cargo run --example read_property_multiple -- --addr "192.168.1.249:47808"
// cargo run --example read_property_multiple --no-default-features -- --addr "192.168.1.249:47808"

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

/// A Bacnet Client example to read specific property values for analog input #1
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
    //let args = Args::parse();
    let args = Args {
        addr: "192.168.1.249:47808".into(),
    };
    let mut bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 1500];

    // fetch and print
    let objects = vec![ReadPropertyMultipleObject::new(
        ObjectId::new(ObjectType::ObjectAnalogInput, 1),
        vec![
            PropertyId::PropObjectName,
            PropertyId::PropPresentValue,
            PropertyId::PropUnits,
            PropertyId::PropStatusFlags,
        ],
    )];
    let request = ReadPropertyMultiple::new(objects);
    let result = bacnet.read_property_multiple(&mut buf, request).await?;
    println!("{:?}", result);
    Ok(())
}

#[cfg(not(feature = "alloc"))]
#[tokio::main]
async fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    let args = Args::parse();
    let mut bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 4096];

    // fetch
    let object_id = ObjectId::new(ObjectType::ObjectAnalogInput, 1);
    let property_ids = [
        PropertyId::PropObjectName,
        PropertyId::PropPresentValue,
        PropertyId::PropUnits,
        PropertyId::PropStatusFlags,
    ];
    let objects = [ReadPropertyMultipleObject::new(object_id, &property_ids)];
    let request = ReadPropertyMultiple::new(&objects);
    let result = bacnet.read_property_multiple(&mut buf, request).await?;

    // inspect results - loop though objects
    for values in &result {
        // print property values of object
        for x in &values?.property_results {
            println!("{:?}", x?);
        }
    }

    Ok(())
}
