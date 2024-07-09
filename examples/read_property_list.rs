// cargo run --example read_property_list -- --addr "192.168.1.249:47808" --device-id 79079
// cargo run --example read_property_list --no-default-features -- --addr "192.168.1.249:47808" --device-id 79079

use clap::{command, Parser};
use common::MySocket;
use embedded_bacnet::{
    application_protocol::services::read_property::{
        ReadProperty, ReadPropertyAck, ReadPropertyValue,
    },
    common::{
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    simple::BacnetError,
};

mod common;

/// A Bacnet Client example to read the list of properties for the device
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

#[tokio::main]
async fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    let args = Args::parse();
    let mut bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 1500];

    // fetch
    let object_id = ObjectId::new(ObjectType::ObjectDevice, args.device_id);
    let request = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let result = bacnet.read_property(&mut buf, request).await?;

    // print
    print_result(result)
}

#[cfg(feature = "alloc")]
fn print_result(result: ReadPropertyAck) -> Result<(), BacnetError<MySocket>> {
    if let ReadPropertyValue::ObjectIdList(list) = result.property_value {
        for item in &list.object_ids {
            println!("{:?}", item);
        }
    }
    Ok(())
}

#[cfg(not(feature = "alloc"))]
fn print_result(result: ReadPropertyAck) -> Result<(), BacnetError<MySocket>> {
    if let ReadPropertyValue::ObjectIdList(list) = result.property_value {
        for item in &list {
            println!("{:?}", item?);
        }
    }

    Ok(())
}
