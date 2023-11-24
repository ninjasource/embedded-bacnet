use core::ops::Range;
use std::net::UdpSocket;

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use embedded_bacnet::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ComplexAck, ComplexAckService, ConfirmedRequest, ConfirmedRequestService},
        primitives::data_value::ApplicationDataValue,
        services::{
            read_property::{ReadProperty, ReadPropertyAck, ReadPropertyValue},
            read_range::{ReadRange, ReadRangeByPosition, ReadRangeRequestType, ReadRangeValue},
        },
    },
    common::{
        io::{Reader, Writer},
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
    },
};

#[derive(Debug)]
enum MainError {
    Io(std::io::Error),
    Bacnet(embedded_bacnet::common::error::Error),
}

impl From<std::io::Error> for MainError {
    fn from(value: std::io::Error) -> Self {
        MainError::Io(value)
    }
}

impl From<embedded_bacnet::common::error::Error> for MainError {
    fn from(value: embedded_bacnet::common::error::Error) -> Self {
        MainError::Bacnet(value)
    }
}

const IP_ADDRESS: &str = "192.168.1.249:47808";

fn main() -> Result<(), MainError> {
    simple_logger::init().unwrap();
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", 0xBAC1))?;
    let object_id = ObjectId::new(ObjectType::ObjectTrendlog, 4);

    let record_count = get_record_count(&socket, object_id.clone())? as usize;
    println!("Record count {record_count}");

    const MAX_LOG_COUNT_PER_REQ: usize = 55;

    let mut log_set = LogSet::new(record_count);

    for row in (1..=record_count).step_by(MAX_LOG_COUNT_PER_REQ) {
        get_items_for_range(
            &socket,
            object_id.clone(),
            row..MAX_LOG_COUNT_PER_REQ,
            &mut log_set,
        )?;
    }

    log_set.finished();

    Ok(())
}

#[derive(Debug, Clone)]
struct LogSet {
    pub level0: Vec<LogEntry>, // every 15 mins
    pub level1: Vec<LogEntry>, // every hour
    pub level2: Vec<LogEntry>, // every 4 hours
    pub level3: Vec<LogEntry>, // every 12 hours

    level1_current: Aggregation,
    level2_current: Aggregation,
    level3_current: Aggregation,
}

impl LogSet {
    pub fn new(record_count: usize) -> Self {
        LogSet {
            level0: Vec::with_capacity(record_count),
            level1: Vec::with_capacity(record_count / 4),
            level2: Vec::with_capacity(record_count / 16),
            level3: Vec::with_capacity(24),
            level1_current: Aggregation::new(Duration::hours(1)),
            level2_current: Aggregation::new(Duration::hours(4)),
            level3_current: Aggregation::new(Duration::hours(12)),
        }
    }

    pub fn add_entry(&mut self, log_entry: LogEntry) {
        self.level0.push(log_entry.clone());

        if let Some(aggregation) = self.level1_current.add_entry(log_entry.clone()) {
            self.level1.push(aggregation);
        }

        if let Some(aggregation) = self.level2_current.add_entry(log_entry.clone()) {
            let date_time =
                NaiveDateTime::from_timestamp_opt(aggregation.timestamp as i64, 0).unwrap();
            println!("{} - {}", date_time, aggregation.value);
            self.level2.push(aggregation);
        }

        if let Some(aggregation) = self.level3_current.add_entry(log_entry) {
            if self.level3.len() != self.level3.capacity() {
                self.level3.push(aggregation);
            }
        }
    }

    pub fn finished(&mut self) {
        if let Some(aggregation) = self.level1_current.finished() {
            self.level1.push(aggregation);
        }
        if let Some(aggregation) = self.level2_current.finished() {
            let date_time =
                NaiveDateTime::from_timestamp_opt(aggregation.timestamp as i64, 0).unwrap();
            println!("{} - {}", date_time, aggregation.value);
            self.level2.push(aggregation);
        }
        if let Some(aggregation) = self.level2_current.finished() {
            if self.level3.len() != self.level3.capacity() {
                self.level3.push(aggregation);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Aggregation {
    duration: Duration,
    date_time: Option<NaiveDateTime>,
    value: f32,
    count: usize,
}

impl Aggregation {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            date_time: None,
            value: 0.0,
            count: 0,
        }
    }

    fn calculate_bucket(&self, dt: &NaiveDateTime) -> NaiveDateTime {
        let hour = dt.time().hour() - (dt.time().hour() % self.duration.num_hours() as u32);
        NaiveDateTime::new(
            NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day()).unwrap(),
            NaiveTime::from_hms_opt(hour, 0, 0).unwrap(),
        )
    }

    pub fn add_entry(&mut self, log_entry: LogEntry) -> Option<LogEntry> {
        let dt = NaiveDateTime::from_timestamp_opt(log_entry.timestamp as i64, 0).unwrap();
        let bucket = self.calculate_bucket(&dt);

        match self.date_time {
            Some(date_time) => {
                if date_time == bucket {
                    self.value += log_entry.value;
                    self.count += 1;
                } else {
                    let agg_log_entry = LogEntry {
                        timestamp: date_time.timestamp() as i32,
                        value: self.value / self.count as f32,
                    };

                    self.date_time = Some(bucket);
                    self.value = log_entry.value;
                    self.count = 1;
                    return Some(agg_log_entry);
                }
            }
            None => {
                self.date_time = Some(bucket);
                self.value = log_entry.value;
                self.count = 1;
            }
        }
        None
    }

    pub fn finished(&self) -> Option<LogEntry> {
        match self.date_time {
            Some(date_time) => Some(LogEntry {
                timestamp: date_time.timestamp() as i32,
                value: self.value / self.count as f32,
            }),
            None => None,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct LogEntry {
    pub timestamp: i32,
    pub value: f32,
}

fn get_items_for_range(
    socket: &UdpSocket,
    object_id: ObjectId,
    range: Range<usize>,
    items: &mut LogSet,
) -> Result<(), MainError> {
    // encode packet
    let request_type = ReadRangeRequestType::ByPosition(ReadRangeByPosition {
        index: range.start as u32,
        count: range.end as u32,
    });
    let rp = ReadRange::new(object_id, PropertyId::PropLogBuffer, request_type);
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::ReadRange(rp));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));
    let mut buffer = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buffer);
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    socket.send_to(buf, IP_ADDRESS)?;

    // receive reply
    let mut buf = vec![0; 4096];
    let (n, _peer) = socket.recv_from(&mut buf)?;
    let buf = &buf[..n];
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf)?;
    let ack: ComplexAck = message.try_into()?;

    match ack.service {
        ComplexAckService::ReadRange(read_range) => {
            for item in &read_range.item_data {
                let item = item?;
                let value = match item.value {
                    ReadRangeValue::Real(x) => x,
                    _ => 0.0,
                };
                let date_time = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(
                        item.date.year as i32,
                        item.date.month as u32,
                        item.date.day as u32,
                    )
                    .unwrap(),
                    NaiveTime::from_hms_opt(item.time.hour as u32, item.time.minute as u32, 0)
                        .unwrap(),
                );
                let log_entry = LogEntry {
                    timestamp: date_time.timestamp() as i32,
                    value,
                };
                items.add_entry(log_entry);
            }
        }
        _ => {
            // do nothing
        }
    }

    Ok(())
}

fn get_record_count(socket: &UdpSocket, object_id: ObjectId) -> Result<u32, MainError> {
    // encode packet
    let rp = ReadProperty::new(object_id, PropertyId::PropRecordCount);
    let req = ConfirmedRequest::new(0, ConfirmedRequestService::ReadProperty(rp));
    let apdu = ApplicationPdu::ConfirmedRequest(req);
    let src = None;
    let dst = None;
    let message = NetworkMessage::Apdu(apdu);
    let npdu = NetworkPdu::new(src, dst, true, MessagePriority::Normal, message);
    let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));
    let mut buffer = vec![0; 16 * 1024];
    let mut buffer = Writer::new(&mut buffer);
    data_link.encode(&mut buffer);

    // send packet
    let buf = buffer.to_bytes();
    socket.send_to(buf, IP_ADDRESS)?;
    println!("Sent:     {:02x?} to {}\n", buf, IP_ADDRESS);

    // receive reply
    let mut buf = vec![0; 1024];
    let (n, peer) = socket.recv_from(&mut buf)?;
    let buf = &buf[..n];
    println!("Received: {:02x?} from {:?}", buf, peer);
    let mut reader = Reader::default();
    let message = DataLink::decode(&mut reader, buf)?;
    println!("Decoded:  {:?}\n", message);
    let message: ReadPropertyAck = message.try_into()?;

    // read values
    if let ReadPropertyValue::ApplicationDataValue(ApplicationDataValue::UnsignedInt(x)) =
        message.property_value
    {
        Ok(x)
    } else {
        Ok(0)
    }
}
