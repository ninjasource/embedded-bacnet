use core::{fmt::Debug, future::Future};

use crate::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{ComplexAck, ComplexAckService, ConfirmedRequest, ConfirmedRequestService},
        services::{
            read_property::{ReadProperty, ReadPropertyAck},
            read_property_multiple::{ReadPropertyMultiple, ReadPropertyMultipleAck},
            read_range::{ReadRange, ReadRangeAck},
            time_synchronization::TimeSynchronization,
            write_property::WriteProperty,
        },
        unconfirmed::UnconfirmedRequest,
    },
    common::{
        error::Error,
        io::{Reader, Writer},
    },
    network_protocol::{
        data_link::{DataLink, DataLinkFunction},
        network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
    },
};

pub trait NetworkIo {
    type Error: Debug;
    fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<usize, Self::Error>> + Send;
    fn write(&mut self, buf: &[u8]) -> impl Future<Output = Result<usize, Self::Error>> + Send;
}

#[derive(Debug)]
pub struct Bacnet<T>
where
    T: NetworkIo + Debug,
{
    io: T,
    invoke_id: u8,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BacnetError<T>
where
    T: NetworkIo,
{
    Io(T::Error),
    Codec(Error),
    InvokeId(InvokeIdError),
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvokeIdError {
    pub expected: u8,
    pub actual: u8,
}

impl<T> Bacnet<T>
where
    T: NetworkIo + Debug,
{
    pub fn new(io: T) -> Self {
        Self { io, invoke_id: 0 }
    }

    pub fn take(self) -> T {
        self.io
    }

    fn get_then_inc_invoke_id(&mut self) -> u8 {
        let invoke_id = self.invoke_id;

        if self.invoke_id == u8::MAX {
            self.invoke_id = 0;
        } else {
            self.invoke_id += 1;
        }

        invoke_id
    }

    async fn send_and_receive<'a>(
        &mut self,
        buf: &'a mut [u8],
        service: ConfirmedRequestService<'_>,
    ) -> Result<ComplexAck<'a>, BacnetError<T>> {
        let invoke_id = self.get_then_inc_invoke_id();
        let apdu = ApplicationPdu::ConfirmedRequest(ConfirmedRequest::new(invoke_id, service));
        let message = NetworkMessage::Apdu(apdu);
        let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
        let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));

        {
            let mut writer = Writer::new(buf);
            data_link.encode(&mut writer);

            // send packet
            let buffer = writer.to_bytes();
            self.io
                .write(buffer)
                .await
                .map_err(|e| BacnetError::Io(e))?;
        }

        // receive reply
        let n = self.io.read(buf).await.map_err(|e| BacnetError::Io(e))?;
        let buf = &buf[..n];

        let mut reader = Reader::default();
        let message = DataLink::decode(&mut reader, buf).map_err(|e| BacnetError::Codec(e))?;

        let ack: ComplexAck = message.try_into().map_err(|e| BacnetError::Codec(e))?;
        if ack.invoke_id != invoke_id {
            return Err(BacnetError::InvokeId(InvokeIdError {
                expected: invoke_id,
                actual: ack.invoke_id,
            }));
        }

        Ok(ack)
    }

    async fn unconfirmed_send(
        &mut self,
        buf: &mut [u8],
        service: UnconfirmedRequest<'_>,
    ) -> Result<(), BacnetError<T>> {
        let apdu = ApplicationPdu::UnconfirmedRequest(service);
        let message = NetworkMessage::Apdu(apdu);
        let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
        let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));

        let mut writer = Writer::new(buf);
        data_link.encode(&mut writer);

        // send packet
        let buffer = writer.to_bytes();
        self.io
            .write(buffer)
            .await
            .map_err(|e| BacnetError::Io(e))?;
        Ok(())
    }

    pub async fn read_property_multiple<'a>(
        &mut self,
        buf: &'a mut [u8],
        request: ReadPropertyMultiple<'_>,
    ) -> Result<ReadPropertyMultipleAck<'a>, BacnetError<T>> {
        let service = ConfirmedRequestService::ReadPropertyMultiple(request);
        let ack = self.send_and_receive(buf, service).await?;
        match ack.service {
            ComplexAckService::ReadPropertyMultiple(ack) => Ok(ack),
            _ => Err(BacnetError::Codec(Error::ConvertDataLink(
                "apdu message is not a ComplexAckService ReadPropertyMultipleAck",
            ))),
        }
    }

    pub async fn read_property<'a>(
        &mut self,
        buf: &'a mut [u8],
        request: ReadProperty,
    ) -> Result<ReadPropertyAck<'a>, BacnetError<T>> {
        let service = ConfirmedRequestService::ReadProperty(request);
        let ack = self.send_and_receive(buf, service).await?;
        match ack.service {
            ComplexAckService::ReadProperty(ack) => Ok(ack),
            _ => Err(BacnetError::Codec(Error::ConvertDataLink(
                "apdu message is not a ComplexAckService ReadPropertyAck",
            ))),
        }
    }

    pub async fn read_range<'a>(
        &mut self,
        buf: &'a mut [u8],
        request: ReadRange,
    ) -> Result<ReadRangeAck<'a>, BacnetError<T>> {
        let service = ConfirmedRequestService::ReadRange(request);
        let ack = self.send_and_receive(buf, service).await?;
        match ack.service {
            ComplexAckService::ReadRange(ack) => Ok(ack),
            _ => Err(BacnetError::Codec(Error::ConvertDataLink(
                "apdu message is not a ComplexAckService ReadRangeAck",
            ))),
        }
    }

    pub async fn write_property<'a>(
        &mut self,
        buf: &'a mut [u8],
        request: WriteProperty<'_>,
    ) -> Result<(), BacnetError<T>> {
        let service = ConfirmedRequestService::WriteProperty(request);
        let _ack = self.send_and_receive(buf, service).await?;
        Ok(())
    }

    pub async fn time_sync(
        &mut self,
        buf: &mut [u8],
        request: TimeSynchronization,
    ) -> Result<(), BacnetError<T>> {
        let service = UnconfirmedRequest::TimeSynchronization(request);
        self.unconfirmed_send(buf, service).await
    }
}
