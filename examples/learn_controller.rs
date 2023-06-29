#![allow(dead_code, unreachable_code, unused_variables)]

use std::{io::Error, net::UdpSocket};

use embedded_bacnet::{
    application_protocol::{
        application_pdu::{ApplicationPdu, ConfirmedRequest, ConfirmedRequestSerivice},
        primitives::data_value::{ApplicationDataValue, Enumerated},
        read_property::ReadProperty,
        read_property_multiple::{PropertyValue, ReadPropertyMultiple, ReadPropertyMultipleObject},
    },
    common::{
        helper::{Buffer, Reader},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
    },
};

// This is a demo application showcasing some of the functionality of this bacnet library

fn main() -> Result<(), Error> {
    simple_logger::init().unwrap();

    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC0))?;

    // encode packet
    let object_id = ObjectId::new(ObjectType::ObjectDevice, 79079);
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
    println!("Received: {:02x?} from {:?}", payload, peer);
    let mut reader = Reader::new(payload);
    let message = DataLink::decode(&mut reader);
    if let Some(ack) = message.get_read_property_ack() {
        // let mut objects = Vec::new();
        for item in &ack.properties {
            match item.object_type {
                ObjectType::ObjectBinaryOutput
                | ObjectType::ObjectBinaryInput
                | ObjectType::ObjectBinaryValue => {
                    let mut property_ids = Vec::new();
                    property_ids.push(PropertyId::PropObjectName);
                    property_ids.push(PropertyId::PropPresentValue);
                    property_ids.push(PropertyId::PropStatusFlags);
                    let rpm = ReadPropertyMultipleObject::new(*item, property_ids);
                    get_multi(rpm, &socket)?;
                }
                ObjectType::ObjectAnalogInput
                | ObjectType::ObjectAnalogOutput
                | ObjectType::ObjectAnalogValue => {
                    let mut property_ids = Vec::new();
                    property_ids.push(PropertyId::PropObjectName);
                    property_ids.push(PropertyId::PropPresentValue);
                    property_ids.push(PropertyId::PropUnits);
                    property_ids.push(PropertyId::PropStatusFlags);
                    let rpm = ReadPropertyMultipleObject::new(*item, property_ids);
                    get_multi(rpm, &socket)?;
                }
                _ => {}
            };

            // any more than this and things go wrong
            // if objects.len() > 18 {
            //     break;
            // }

            // break;
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

fn get_multi_all(rpm: ReadPropertyMultipleObject, socket: &UdpSocket) -> Result<(), Error> {
    let rpm = ReadPropertyMultiple::new(vec![rpm]);
    //println!("len: {} {:?}", rpm.objects.len(), rpm);
    let buffer = read_property_multiple_to_bytes(rpm);
    let addr = format!("192.168.1.249:{}", 0xBAC0);
    socket.send_to(buffer.to_bytes(), &addr)?;

    let mut buf = vec![0; 1024];

    let (n, peer) = socket.recv_from(&mut buf).unwrap();
    let payload = &buf[..n];
    let mut reader = Reader::new(payload);
    let message = DataLink::decode(&mut reader);

    if let Some(ack) = message.get_read_property_multiple_ack() {
        println!("{:?}", ack);
    }

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
        //return Ok(());
        let object = &ack.objects[0];
        match object.results.len() {
            3 => {
                let value = match &object.results[1].value {
                    PropertyValue::PropValue(ApplicationDataValue::Enumerated(
                        Enumerated::Binary(x),
                    )) => format!("{x:?}"),
                    x => format!("{x:?}"),
                };
                println!(
                    "{:?}({}) {}: {}",
                    object.object_id.object_type,
                    object.object_id.id,
                    object.results[0].value,
                    value
                );
            }
            4 => {
                let units = match &object.results[2].value {
                    PropertyValue::PropValue(ApplicationDataValue::Enumerated(
                        Enumerated::Units(x),
                    )) => format!("{x:?}"),
                    _ => format!(""),
                };
                println!(
                    "{:?}({}) {}: {} {}",
                    object.object_id.object_type,
                    object.object_id.id,
                    object.results[0].value,
                    object.results[1].value,
                    units
                );
            }
            _ => {
                println!("none")
            }
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
