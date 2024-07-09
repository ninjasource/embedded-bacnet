// use this example to troubleshoot an invalid BACnet packet
// RUST_BACKTRACE=1 cargo run --example troubleshoot

use embedded_bacnet::{common::io::Reader, network_protocol::data_link::DataLink};

fn main() {
    // a valid BACnet packet
    let buf = vec![
        129, 10, 0, 21, 1, 4, 2, 117, 1, 14, 12, 2, 0, 3, 243, 30, 9, 56, 9, 57, 31,
    ];

    // use the DataLink codec to decode the bytes
    let mut reader = Reader::default();

    // if the packet is invalid this should panid
    let message = DataLink::decode(&mut reader, &buf).unwrap();

    println!("{:?}", message)
}
