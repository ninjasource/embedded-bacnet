/// A Bacnet Client blocking (non-async) example to read specific property values for analog input #1
// cargo run --example read_property_multiple_blocking --features="is_sync" -- --addr "0.0.0.0:47808"
use std::{io, net::UdpSocket};

use clap::{command, Parser};
use embedded_bacnet::{
    application_protocol::services::read_property_multiple::{
        ReadPropertyMultiple, ReadPropertyMultipleObject,
    },
    common::{
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    simple::{Bacnet, BacnetError, NetworkIo},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address with port e.g. "192.168.1.249:47808"
    #[arg(short, long)]
    addr: String,
}

fn get_bacnet_socket(addr: &str) -> Result<Bacnet<MySocket>, BacnetError<MySocket>> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC1)).map_err(|e| BacnetError::Io(e))?;
    socket.connect(addr).map_err(|e| BacnetError::Io(e))?;
    let socket = MySocket::new(socket);
    Ok(Bacnet::new(socket))
}

#[derive(Debug)]
pub struct MySocket {
    socket: UdpSocket,
}

impl MySocket {
    pub fn new(socket: UdpSocket) -> Self {
        Self { socket }
    }
}

impl NetworkIo for MySocket {
    type Error = io::Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.socket.recv(buf)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.socket.send(buf)
    }
}

fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    simple_logger::init().unwrap();
    let args = Args::parse();
    let mut bacnet = get_bacnet_socket(&args.addr)?;
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
    let result = bacnet.read_property_multiple(&mut buf, request)?;

    // print
    for values in &result {
        for x in &values?.property_results {
            println!("{:?}", x?);
        }
    }

    Ok(())
}
