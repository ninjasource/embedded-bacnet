#![allow(dead_code, unreachable_code, unused_variables)]

use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::{
            ApplicationPdu, ConfirmedRequest, ConfirmedRequestSerivice, UnconfirmedRequest,
        },
        read_property::ReadProperty,
        read_property_multiple::{ReadPropertyMultiple, ReadPropertyMultipleObject},
        who_is::WhoIs,
    },
    common::{
        helper::{Buffer, Reader},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{DestinationAddress, MessagePriority, NetworkMessage, NetworkPdu},
    },
};

// This is a demo application showcasing some of the functionality of this bacnet library

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();

    // broadcast_who_is()
    // read_property_list()
    // read_property_multiple()
    // read_property_multiple_all()
    learn_controller()
}

fn learn_controller() -> Result<(), Error> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectDevice, 20088);
    let read_property = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadProperty(read_property));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu(npdu));
    let mut buffer = Buffer::new();
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    // receive reply
    let mut buf = vec![0; 64 * 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let payload = &buf[..n];
    //println!("Received: {:02x?} from {:?}", payload, peer);
    let mut reader = Reader::new(payload);
    let message = DataLink::decode(&mut reader);
    if let Some(ack) = message.get_read_property_ack() {
        // let mut objects = Vec::new();
        for item in &ack.properties {
            match item.object_type {
                ObjectType::ObjectBinaryOutput | ObjectType::ObjectBinaryInput => {
                    let mut property_ids = Vec::new();
                    property_ids.push(PropertyId::PropObjectName);
                    property_ids.push(PropertyId::PropPresentValue);
                    let rpm = ReadPropertyMultipleObject::new(*item, property_ids);

                    get_multi(rpm, &socket)?;
                }
                ObjectType::ObjectAnalogInput | ObjectType::ObjectAnalogOutput => {
                    let mut property_ids = Vec::new();
                    property_ids.push(PropertyId::PropObjectName);
                    property_ids.push(PropertyId::PropPresentValue);
                    property_ids.push(PropertyId::PropUnits);
                    let rpm = ReadPropertyMultipleObject::new(*item, property_ids);

                    get_multi(rpm, &socket)?;
                }
                _ => {}
            };

            // any more than this and things go wrong
            // if objects.len() > 18 {
            //     break;
            // }
        }

        /*
        let rpm = ReadPropertyMultiple::new(objects);
        //println!("len: {} {:?}", rpm.objects.len(), rpm);
        let buffer = read_property_multiple_to_bytes(rpm);

        socket.send_to(buffer.to_bytes(), &addr)?;

        let (n, peer) = socket.recv_from(&mut buf).unwrap();
        let payload = &buf[..n];
        //println!("Received: {:02x?} from {:?}", payload, peer);
        let mut reader = Reader::new(payload);
        let message = DataLink::decode(&mut reader);
        println!("{message:?}");
        */
    }

    //println!("Decoded:  {:?}\n", message);

    Ok(())
}

fn get_multi(rpm: ReadPropertyMultipleObject, socket: &UdpSocket) -> Result<(), Error> {
    let rpm = ReadPropertyMultiple::new(vec![rpm]);
    //println!("len: {} {:?}", rpm.objects.len(), rpm);
    let buffer = read_property_multiple_to_bytes(rpm);
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buffer.to_bytes(), &addr)?;

    let mut buf = vec![0; 1024];

    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let payload = &buf[..n];
    //println!("Received: {:02x?} from {:?}", payload, peer);
    let mut reader = Reader::new(payload);
    let message = DataLink::decode(&mut reader);

    if let Some(ack) = message.get_read_property_multiple_ack() {
        //println!("{:?}", ack);
        let object = &ack.objects[0];
        match object.results.len() {
            2 => println!("{}: {}", object.results[0].value, object.results[1].value),
            3 => println!(
                "{}: {} {}",
                object.results[0].value, object.results[1].value, object.results[2].value
            ),
            _ => {}
        }
        //  ack.objects[0].results[0].value
    }

    Ok(())
}

struct AnalogInputOutput {
    pub id: u32,
    pub name: String,
    pub unit: String,
    pub value: f32,
}

struct DigitalInputOutput {
    pub id: u32,
    pub name: String,
    pub value: bool,
}

struct Controller {
    pub analog_inputs: Vec<AnalogInputOutput>,
    pub analog_outputs: Vec<AnalogInputOutput>,
    pub digital_inputs: Vec<DigitalInputOutput>,
    pub digital_outputs: Vec<DigitalInputOutput>,
}

fn read_property_multiple_to_bytes(rpm: ReadPropertyMultiple) -> Buffer {
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadPropertyMultiple(rpm));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu(npdu));
    let mut buffer = Buffer::new();
    data_link.encode(&mut buffer);
    buffer
}

fn read_property_multiple() -> Result<(), Error> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectAnalogInput, 2);
    let mut property_ids = Vec::new();
    property_ids.push(PropertyId::PropPresentValue);
    let rpm = ReadPropertyMultipleObject::new(object_id, property_ids);
    let rpm = ReadPropertyMultiple::new(vec![rpm]);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadPropertyMultiple(rpm));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu(npdu));
    let mut buffer = Buffer::new();
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let payload = &buf[..n];
    println!("Received: {:02x?} from {:?}", payload, peer);
    let mut reader = Reader::new(payload);
    let message = DataLink::decode(&mut reader);
    println!("Decoded:  {:?}\n", message);

    Ok(())
}

fn read_property_multiple_all() -> Result<(), Error> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectAnalogInput, 1);
    let mut property_ids = Vec::new();
    property_ids.push(PropertyId::PropAll);
    let rpm = ReadPropertyMultipleObject::new(object_id, property_ids);
    let rpm = ReadPropertyMultiple::new(vec![rpm]);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadPropertyMultiple(rpm));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu(npdu));
    let mut buffer = Buffer::new();
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let payload = &buf[..n];
    println!("Received: {:02x?} from {:?}", payload, peer);
    let mut reader = Reader::new(payload);
    let message = DataLink::decode(&mut reader);
    println!("Decoded:  {:?}\n", message);

    Ok(())
}

fn read_property_list() -> Result<(), Error> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectDevice, 20088);
    let read_property = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let req = ConfirmedRequest::new(0, ConfirmedRequestSerivice::ReadProperty(read_property));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu(npdu));
    let mut buffer = Buffer::new();
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let payload = &buf[..n];
    println!("Received: {:02x?} from {:?}", payload, peer);
    let mut reader = Reader::new(payload);
    let message = DataLink::decode(&mut reader);
    println!("Decoded:  {:?}\n", message);

    Ok(())
}

fn broadcast_who_is() -> Result<(), Error> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;
    socket.set_broadcast(true)?;

    let who_is = WhoIs {};
    let apdu = ApplicationPdu::UnconfirmedRequest(UnconfirmedRequest::WhoIs(who_is));
    let src = None;
    let dst = Some(DestinationAddress::new(0xffff, None));
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, false, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalBroadcastNpdu(npdu));

    let mut buffer = Buffer::new();
    data_link.encode(&mut buffer);

    let buf = buffer.to_bytes();
    let addr = format!("255.255.255.255:{}", 0xBAC0);
    socket.send_to(buf, &addr)?;
    println!("Sent:     {:02x?} to {}\n", buf, addr);

    let mut buf = vec![0; 1024];
    loop {
        let (n, peer) = socket.recv_from(&mut buf).unwrap();
        let payload = &buf[..n];
        println!("Received: {:02x?} from {:?}", payload, peer);
        let mut reader = Reader::new(payload);
        let message = DataLink::decode(&mut reader);
        println!("Decoded:  {:?}\n", message);
    }
}
