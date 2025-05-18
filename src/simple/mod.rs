/// This module is meant to be a very basic way to interact with a BACnet IP network in a simple request / response manner
/// It automatically links up requests with responses using an invoke_id which only really works when you send one request at a time.
/// If you intend to fire off many simultaneous requests then you should keep track of invoke_ids and handle congestion and packet ordering yourself.
/// Your NetworkIo implementation is responsible for timeout detection for reads and writes.
/// This is an async-first module but you can run it in a native blocking way if you like.
///   The `maybe_async` crate is used to avoid code duplication and completely stips away async code when the `is_sync` feature flag is set.
/// If you are having trouble with the borrow checker try enabling the `alloc` feature to make BACnet objects fully owned
use core::{
    fmt::Debug,
    sync::atomic::{AtomicU8, Ordering},
};

use maybe_async::maybe_async;

use crate::{
    application_protocol::{
        application_pdu::ApplicationPdu,
        confirmed::{
            ComplexAck, ComplexAckService, ConfirmedRequest, ConfirmedRequestService, SimpleAck,
        },
        services::{
            change_of_value::{CovNotification, SubscribeCov},
            i_am::IAm,
            read_property::{ReadProperty, ReadPropertyAck},
            read_property_multiple::{ReadPropertyMultiple, ReadPropertyMultipleAck},
            read_range::{ReadRange, ReadRangeAck},
            time_synchronization::TimeSynchronization,
            who_is::WhoIs,
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
        network_pdu::{DestinationAddress, MessagePriority, NetworkMessage, NetworkPdu},
    },
};

#[derive(Debug)]
pub struct Bacnet<T>
where
    T: NetworkIo + Debug,
{
    pub io: T,
    invoke_id: AtomicU8,
}

#[allow(async_fn_in_trait)]
#[cfg(feature = "defmt")]
#[maybe_async(AFIT)] // AFIT - Async Function In Trait
pub trait NetworkIo {
    type Error: Debug + defmt::Format;
    async fn read(&self, buf: &mut [u8]) -> Result<usize, Self::Error>;
    async fn write(&self, buf: &[u8]) -> Result<usize, Self::Error>;
}

#[cfg(not(feature = "defmt"))]
#[allow(async_fn_in_trait)]
#[maybe_async(AFIT)] // AFIT - Async Function In Trait
pub trait NetworkIo {
    type Error: Debug;

    async fn read(&self, buf: &mut [u8]) -> Result<usize, Self::Error>;
    async fn write(&self, buf: &[u8]) -> Result<usize, Self::Error>;
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

impl<T: NetworkIo> From<Error> for BacnetError<T> {
    fn from(value: Error) -> Self {
        Self::Codec(value)
    }
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
        Self {
            io,
            invoke_id: AtomicU8::new(0),
        }
    }

    /// Returns the socket back to the caller and consumes self
    pub fn into_inner(self) -> T {
        self.io
    }

    #[maybe_async()]
    pub async fn who_is(&self, buf: &mut [u8]) -> Result<Option<IAm>, BacnetError<T>> {
        let apdu = ApplicationPdu::UnconfirmedRequest(UnconfirmedRequest::WhoIs(WhoIs {}));
        let dst = Some(DestinationAddress::new(0xffff, None));
        let message = NetworkMessage::Apdu(apdu);
        let npdu = NetworkPdu::new(None, dst, false, MessagePriority::Normal, message);
        let data_link = DataLink::new(DataLinkFunction::OriginalBroadcastNpdu, Some(npdu));

        let mut writer = Writer::new(buf);
        data_link.encode(&mut writer);

        // send packet until we get a reply
        let buffer = writer.to_bytes();

        self.io.write(buffer).await.map_err(BacnetError::Io)?;

        // receive reply
        let n = self.io.read(buf).await.map_err(BacnetError::Io)?;
        let buf = &buf[..n];

        // use the DataLink codec to decode the bytes
        let mut reader = Reader::default();
        let message = DataLink::decode(&mut reader, buf).map_err(BacnetError::Codec)?;

        if let Some(npdu) = message.npdu {
            if let NetworkMessage::Apdu(ApplicationPdu::UnconfirmedRequest(
                UnconfirmedRequest::IAm(iam),
            )) = npdu.network_message
            {
                return Ok(Some(iam));
            }
        };

        Ok(None)
    }

    #[maybe_async()]
    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub async fn read_property_multiple<'a>(
        &self,
        buf: &'a mut [u8],
        request: ReadPropertyMultiple<'_>,
    ) -> Result<ReadPropertyMultipleAck<'a>, BacnetError<T>> {
        let service = ConfirmedRequestService::ReadPropertyMultiple(request);
        let ack = self.send_and_receive_complex_ack(buf, service).await?;
        match ack.service {
            ComplexAckService::ReadPropertyMultiple(ack) => Ok(ack),
            _ => Err(BacnetError::Codec(Error::ConvertDataLink(
                "apdu message is not a ComplexAckService ReadPropertyMultipleAck",
            ))),
        }
    }

    #[maybe_async()]
    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub async fn read_property<'a>(
        &self,
        buf: &'a mut [u8],
        request: ReadProperty,
    ) -> Result<ReadPropertyAck<'a>, BacnetError<T>> {
        let service = ConfirmedRequestService::ReadProperty(request);
        let ack = self.send_and_receive_complex_ack(buf, service).await?;
        match ack.service {
            ComplexAckService::ReadProperty(ack) => Ok(ack),
            _ => Err(BacnetError::Codec(Error::ConvertDataLink(
                "apdu message is not a ComplexAckService ReadPropertyAck",
            ))),
        }
    }

    #[maybe_async()]
    pub async fn subscribe_change_of_value(
        &self,
        buf: &mut [u8],
        request: SubscribeCov,
    ) -> Result<(), BacnetError<T>> {
        let service = ConfirmedRequestService::SubscribeCov(request);
        let _ack = self.send_and_receive_simple_ack(buf, service).await?;
        Ok(())
    }

    #[maybe_async()]
    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub async fn read_change_of_value<'a>(
        &self,
        buf: &'a mut [u8],
    ) -> Result<Option<CovNotification<'a>>, BacnetError<T>> {
        let n = self.io.read(buf).await.map_err(BacnetError::Io)?;
        let mut reader = Reader::default();
        let message = DataLink::decode(&mut reader, &buf[..n])?;

        if let Some(npdu) = message.npdu {
            if let NetworkMessage::Apdu(ApplicationPdu::UnconfirmedRequest(
                UnconfirmedRequest::CovNotification(x),
            )) = npdu.network_message
            {
                return Ok(Some(x));
            }
        };

        Ok(None)
    }

    #[maybe_async()]
    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub async fn read_range<'a>(
        &self,
        buf: &'a mut [u8],
        request: ReadRange,
    ) -> Result<ReadRangeAck<'a>, BacnetError<T>> {
        let service = ConfirmedRequestService::ReadRange(request);
        let ack = self.send_and_receive_complex_ack(buf, service).await?;
        match ack.service {
            ComplexAckService::ReadRange(ack) => Ok(ack),
            _ => Err(BacnetError::Codec(Error::ConvertDataLink(
                "apdu message is not a ComplexAckService ReadRangeAck",
            ))),
        }
    }

    #[maybe_async()]
    pub async fn write_property<'a>(
        &self,
        buf: &mut [u8],
        request: WriteProperty<'_>,
    ) -> Result<(), BacnetError<T>> {
        let service = ConfirmedRequestService::WriteProperty(request);
        let _ack = self.send_and_receive_simple_ack(buf, service).await?;
        Ok(())
    }

    #[maybe_async()]
    pub async fn time_sync(
        &self,
        buf: &mut [u8],
        request: TimeSynchronization,
    ) -> Result<(), BacnetError<T>> {
        let service = UnconfirmedRequest::TimeSynchronization(request);
        self.send_unconfirmed(buf, service).await
    }

    #[maybe_async()]
    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    async fn send_and_receive_complex_ack<'a>(
        &self,
        buf: &'a mut [u8],
        service: ConfirmedRequestService<'_>,
    ) -> Result<ComplexAck<'a>, BacnetError<T>> {
        let invoke_id = self.send_confirmed(buf, service).await?;

        loop {
            // receive reply
            let n = self.io.read(buf).await.map_err(BacnetError::Io)?;
            let buf = &buf[..n];

            // use the DataLink codec to decode the bytes
            let mut reader = Reader::default();
            let message = DataLink::decode(&mut reader, buf).map_err(BacnetError::Codec)?;

            match message.npdu {
                Some(x) => match x.network_message {
                    NetworkMessage::Apdu(ApplicationPdu::ComplexAck(ack)) => {
                        // ignore earier messages
                        if ack.invoke_id < invoke_id {
                            continue;
                        }

                        // return message is expected to have the same invoke_id as the request (return error if later invoke id)
                        Self::check_invoke_id(invoke_id, ack.invoke_id)?;
                        return Ok(ack);
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }
    }

    /*

        #[maybe_async()]
    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    async fn send_and_receive_complex_ack<'a>(
        &self,
        buf: &'a mut [u8],
        service: ConfirmedRequestService<'_>,
    ) -> Result<ComplexAck<'a>, BacnetError<T>> {
        let invoke_id = self.send_confirmed(buf, service).await?;

        for _ in 1..5 {
            // receive reply
            let n = self.io.read(buf).await.map_err(BacnetError::Io)?;
            let buf = &buf[..n];

            // use the DataLink codec to decode the bytes
            let mut reader = Reader::default();
            let message = DataLink::decode(&mut reader, buf).map_err(BacnetError::Codec)?;

            match message.npdu {
                Some(x) => match x.network_message {
                    NetworkMessage::Apdu(ApplicationPdu::ComplexAck(ack)) => {
                        // return message is expected to have the same invoke_id as the request
                        Self::check_invoke_id(invoke_id, ack.invoke_id)?;
                        return Ok(ack),
                    }
                    NetworkMessage::Apdu(ApplicationPdu::UnconfirmedRequest()) => {
                        continue;
                    }
                    _ => return Err(Error::ConvertDataLink(
                        "npdu message is not an apdu complex ack",
                    )),
                },
                _ => return Err(Error::ConvertDataLink("no npdu defined in message")),
            }
        }

        Ok(ack)
    }
     */

    #[maybe_async()]
    async fn send_and_receive_simple_ack<'a>(
        &self,
        buf: &mut [u8],
        service: ConfirmedRequestService<'_>,
    ) -> Result<SimpleAck, BacnetError<T>> {
        let invoke_id = self.send_confirmed(buf, service).await?;

        // receive reply
        let n = self.io.read(buf).await.map_err(BacnetError::Io)?;
        let buf = &buf[..n];

        // use the DataLink codec to decode the bytes
        let mut reader = Reader::default();
        let message = DataLink::decode(&mut reader, buf).map_err(BacnetError::Codec)?;

        // TODO: return bacnet error if the server returns one
        // return message is expected to be a ComplexAck
        let ack: SimpleAck = message.try_into().map_err(BacnetError::Codec)?;

        // return message is expected to have the same invoke_id as the request
        Self::check_invoke_id(invoke_id, ack.invoke_id)?;

        Ok(ack)
    }

    #[maybe_async()]
    async fn send_unconfirmed(
        &self,
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
        self.io.write(buffer).await.map_err(BacnetError::Io)?;
        Ok(())
    }

    #[maybe_async()]
    async fn send_confirmed(
        &self,
        buf: &mut [u8],
        service: ConfirmedRequestService<'_>,
    ) -> Result<u8, BacnetError<T>> {
        let invoke_id = self.get_then_inc_invoke_id();
        let apdu = ApplicationPdu::ConfirmedRequest(ConfirmedRequest::new(invoke_id, service));
        let message = NetworkMessage::Apdu(apdu);
        let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
        let data_link = DataLink::new(DataLinkFunction::OriginalUnicastNpdu, Some(npdu));

        let mut writer = Writer::new(buf);
        data_link.encode(&mut writer);

        // send packet
        let buffer = writer.to_bytes();
        self.io.write(buffer).await.map_err(BacnetError::Io)?;

        Ok(invoke_id)
    }

    fn check_invoke_id(expected: u8, actual: u8) -> Result<(), BacnetError<T>> {
        if expected != actual {
            Err(BacnetError::InvokeId(InvokeIdError { expected, actual }))
        } else {
            Ok(())
        }
    }

    fn get_then_inc_invoke_id(&self) -> u8 {
        self.invoke_id.fetch_add(1, Ordering::SeqCst)
    }
}
