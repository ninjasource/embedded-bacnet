use crate::common::{error::Error, io::Reader};

use super::{
    application_pdu::{ApduType, MaxAdpu, MaxSegments},
    confirmed::{ComplexAckService, ConfirmedRequestService, ConfirmedServiceChoice},
};

#[derive(Debug, Clone)]
pub struct Segment<'a> {
    pub apdu_type: ApduType,
    pub more_follows: bool,
    pub max_response_segments: MaxSegments,
    pub max_apdu_size: MaxAdpu,

    pub invoke_id: u8,
    pub sequence_number: u8,
    pub proposed_window_size: u8, // number of segments before an Segment-ACK must be sent
    pub service_choice: u8,

    // apdu data
    pub data: &'a [u8],
}

impl<'a> Segment<'a> {
    pub fn decode(
        more_follows: bool,
        apdu_type: ApduType,
        reader: &mut Reader,
        buf: &'a [u8],
    ) -> Result<Self, Error> {
        let byte0 = reader.read_byte(buf)?;
        let max_segments: MaxSegments = (byte0 & 0xF0).into();
        let max_adpu: MaxAdpu = (byte0 & 0x0F).into();
        let invoke_id = reader.read_byte(buf)?;
        let sequence_number = reader.read_byte(buf)?;
        let proposed_window_size = reader.read_byte(buf)?;
        let service_choice = reader.read_byte(buf)?;

        Ok(Segment {
            apdu_type,
            more_follows,
            max_response_segments: max_segments,
            max_apdu_size: max_adpu,

            invoke_id,
            sequence_number,
            proposed_window_size,
            service_choice,

            data: buf,
        })
    }
}

pub enum CombinedSegments<'a> {
    ConfirmedRequest(u8, ConfirmedRequestService<'a>),
    ComplexAck(u8, ComplexAckService<'a>),
}

// combine and decode a number of segments into it's service.
pub fn decode<'a>(buf: &'a mut [u8], segments: &[Segment]) -> Result<CombinedSegments<'a>, Error> {
    if segments.len() == 0 {
        return Err(Error::InvalidValue("no segments to decode"));
    }

    // TODO: combine all these iterations into one.
    let first_segment = &segments[0];
    if segments
        .iter()
        .all(|segment| segment.apdu_type == first_segment.apdu_type)
        == false
    {
        return Err(Error::InvalidValue(
            "not all segments have matching apdu type",
        ));
    }
    if segments
        .iter()
        .all(|segment| segment.service_choice == first_segment.service_choice)
        == false
    {
        return Err(Error::InvalidValue(
            "not all segments have matching apdu service choice",
        ));
    }
    if segments
        .iter()
        .all(|segment| segment.invoke_id == first_segment.invoke_id)
        == false
    {
        return Err(Error::InvalidValue(
            "not all segments have matching invoke id",
        ));
    }

    // copy segments into buffer including bounds checking
    let total_len: usize = segments.iter().map(|segment| segment.data.len()).sum();
    if total_len > buf.len() {
        return Err(Error::Length((
            "buf not large enough to combine all segments",
            total_len as u32,
        )));
    }

    let mut dest_iter = buf.iter_mut();
    for segment in segments.iter().map(|segment| segment.data) {
        for &byte in segment {
            if let Some(dest_slot) = dest_iter.next() {
                *dest_slot = byte;
            }
        }
    }

    let mut reader = Reader::new_with_len(total_len);
    match first_segment.apdu_type {
        ApduType::ConfirmedServiceRequest => {
            let choice: ConfirmedServiceChoice =
                first_segment.service_choice.try_into().map_err(|e| {
                    Error::InvalidVariant((
                        "ConfirmedRequest decode ConfirmedServiceChoice",
                        e as u32,
                    ))
                })?;
            let service = ConfirmedRequestService::decode(choice, &mut reader, buf)?;
            Ok(CombinedSegments::ConfirmedRequest(
                first_segment.invoke_id,
                service,
            ))
        }
        ApduType::ComplexAck => {
            let choice: ConfirmedServiceChoice =
                first_segment.service_choice.try_into().map_err(|e| {
                    Error::InvalidVariant((
                        "ConfirmedRequest decode ConfirmedServiceChoice",
                        e as u32,
                    ))
                })?;
            let service = ComplexAckService::decode(choice, &mut reader, buf)?;
            Ok(CombinedSegments::ComplexAck(
                first_segment.invoke_id,
                service,
            ))
        }
        _ => unimplemented!(),
    }
}
