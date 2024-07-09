// cargo run --example read_property -- --addr "192.168.1.249:47808"
// cargo run --example read_property --no-default-features -- --addr "192.168.1.249:47808"

use clap::{command, Parser};
use common::MySocket;
use embedded_bacnet::{
    application_protocol::{
        primitives::data_value::ApplicationDataValue,
        services::read_property::{ReadProperty, ReadPropertyValue},
    },
    common::{
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    simple::BacnetError,
};

mod common;

/// A Bacnet Client example to read a property from analog input #1
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

    // fetch
    let object_id = ObjectId::new(ObjectType::ObjectAnalogInput, 1);
    let request = ReadProperty::new(object_id, PropertyId::PropPresentValue);
    let result = bacnet.read_property(&mut buf, request).await?;

    // print
    if let ReadPropertyValue::ApplicationDataValue(ApplicationDataValue::Real(value)) =
        result.property_value
    {
        println!("Value: {:?}", value);
    } else {
        println!("Enexpected value type returned: {:?}", result);
    }

    Ok(())
}
