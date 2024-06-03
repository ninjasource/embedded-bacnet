// cargo run --example who_is -- --addr "192.168.1.249:47808"

use clap::{command, Parser};
use common::MySocket;
use embedded_bacnet::{application_protocol::services::who_is::WhoIs, simple::BacnetError};

mod common;

/// A Bacnet Client example to send a who_is to a specific controller
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
    let request = WhoIs {};
    let result = bacnet.who_is(&mut buf, request).await?;

    // print
    println!("{:?}", result);
    Ok(())
}
