// cargo run --example change_of_value -- --addr "192.168.1.249:47808"

use clap::{command, Parser};
use common::MySocket;
use embedded_bacnet::{
    application_protocol::services::change_of_value::SubscribeCov,
    common::object_id::{ObjectId, ObjectType},
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
    let mut buf = vec![0; 4096];

    // subscribe
    let object_id = ObjectId::new(ObjectType::ObjectAnalogInput, 4);
    let request = SubscribeCov::new(1, object_id, false, 5);
    bacnet.subscribe_change_of_value(&mut buf, request).await?;

    // fetch next
    let result = bacnet.read_change_of_value(&mut buf).await?;

    // print
    if let Some(notification) = result {
        println!("{:?}", notification);
        for property in &notification.values {
            println!("Value: {:?}", property?)
        }
    }

    Ok(())
}
