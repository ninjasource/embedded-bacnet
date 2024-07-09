// cargo run --example write_property -- --addr "192.168.1.249:47808"
// cargo run --example write_property --no-default-features -- --addr "192.168.1.249:47808"

use clap::{command, Parser};
use common::MySocket;
use embedded_bacnet::{
    application_protocol::{
        primitives::data_value::{ApplicationDataValueWrite, Enumerated},
        services::write_property::WriteProperty,
    },
    common::{
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
        spec::Binary,
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
    let mut buf = vec![0; 1500];

    // write
    let request = WriteProperty::new(
        ObjectId::new(ObjectType::ObjectBinaryValue, 3),
        PropertyId::PropPresentValue,
        None,
        None,
        ApplicationDataValueWrite::Enumerated(Enumerated::Binary(Binary::On)),
    );
    bacnet.write_property(&mut buf, request).await?;
    println!("Write ON to BinaryValue no. 3 successful");

    Ok(())
}
