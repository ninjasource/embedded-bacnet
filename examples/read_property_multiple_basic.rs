// cargo run --example read_property_multiple_basic -- --addr "0.0.0.0:47808"

use clap::{command, Parser};
use core::usize;
use embedded_bacnet::{
    application_protocol::services::read_property_multiple::{
        ReadPropertyMultiple, ReadPropertyMultipleObject,
    },
    common::{
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    simple::{Bacnet, NetworkIo},
};
use std::io;
use tokio::net::UdpSocket;

#[derive(Debug)]
struct MySocket {
    socket: UdpSocket,
}

impl MySocket {
    pub fn new(socket: UdpSocket) -> Self {
        Self { socket }
    }
}

impl NetworkIo for MySocket {
    type Error = io::Error;

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.socket.recv(buf).await
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.socket.send(buf).await
    }
}

/// A Bacnet Client example to read specific property values for analog input #1
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address with port e.g. "192.168.1.249:47808"
    #[arg(short, long)]
    addr: String,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    simple_logger::init().unwrap();
    let args = Args::parse();

    let socket = UdpSocket::bind("0.0.0.0:8080").await.unwrap();
    socket.connect(args.addr).await.unwrap();
    let socket = MySocket::new(socket);
    let mut bacnet = Bacnet::new(socket);
    let mut buf = [0; 4096];

    let object_id = ObjectId::new(ObjectType::ObjectAnalogInput, 1);
    let var_name = [
        PropertyId::PropObjectName,
        PropertyId::PropPresentValue,
        PropertyId::PropUnits,
        PropertyId::PropStatusFlags,
    ];
    let property_ids = var_name;
    let objects = [ReadPropertyMultipleObject::new(object_id, &property_ids)];
    let request = ReadPropertyMultiple::new(&objects);
    let result = bacnet
        .read_property_multiple(&mut buf, request)
        .await
        .unwrap();

    for values in &result {
        let values = values.unwrap();
        for x in &values.property_results {
            println!("{:?}", x);
        }
    }

    Ok(())
}
