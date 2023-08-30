fn main() {}

/*
use std::{io, net::UdpSocket};

use embedded_bacnet::common::{
    error::Error,
    helper::{BacnetService, ReadWrite},
    property_id::PropertyId,
};

const IP_ADDRESS: &str = "192.168.1.249:47808";
const DEVICE_ID: u32 = 79079;

fn main() -> Result<(), io::Error> {
    simple_logger::init().unwrap();

    let io = ReadWriteImpl::new(IP_ADDRESS.into())?;
    let mut bacnet = BacnetService::new(io, DEVICE_ID);
    let mut buf = vec![0; 16 * 1024];
    let name = bacnet
        .read_string(PropertyId::PropObjectName, &mut buf)
        .unwrap();

    println!("Name: {name}");

    Ok(())
}

struct ReadWriteImpl {
    socket: UdpSocket,
    remote_addr: String,
}

impl ReadWriteImpl {
    pub fn new(remote_addr: String) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;
        Ok(Self {
            socket,
            remote_addr,
        })
    }
}

impl ReadWrite for ReadWriteImpl {
    fn recv(&self, buf: &mut [u8]) -> Result<usize, Error> {
        let (n, remote_addr) = self.socket.recv_from(buf).map_err(|_| Error::Io)?;
        if self.remote_addr != remote_addr.to_string() {
            panic!("received udp packet from unexpected endpoint");
        }

        Ok(n)
    }

    fn send(&self, buf: &[u8]) -> Result<(), Error> {
        let n = self
            .socket
            .send_to(buf, &self.remote_addr)
            .map_err(|_| Error::Io)?;

        if n != buf.len() {
            panic!("buf too large to fit in a single UDP packet");
        }

        Ok(())
    }
}
*/
