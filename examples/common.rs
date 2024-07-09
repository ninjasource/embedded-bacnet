#![allow(dead_code)]
fn main() {
    // dummy main because this "example" is used for common code for all examples
}

use core::usize;
use embedded_bacnet::simple::{Bacnet, BacnetError, NetworkIo};
use std::io;
use tokio::net::UdpSocket;

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

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.socket.recv(buf).await
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.socket.send(buf).await
    }
}

pub async fn get_bacnet_socket(addr: &str) -> Result<Bacnet<MySocket>, BacnetError<MySocket>> {
    let socket = UdpSocket::bind("0.0.0.0:8080")
        .await
        .map_err(BacnetError::Io)?;
    socket.connect(addr).await.map_err(BacnetError::Io)?;
    let socket = MySocket::new(socket);
    Ok(Bacnet::new(socket))
}
