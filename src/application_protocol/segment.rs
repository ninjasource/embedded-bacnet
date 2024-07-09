// TODO: create an example for this as it is not clear how it should be used in practice
//   Especially because of the "special case" function in there

#[cfg(feature = "alloc")]
use {
    crate::common::spooky::Phantom,
    alloc::{vec, vec::Vec},
};

use crate::{
    application_protocol::application_pdu::{ApduType, PduFlags},
    common::{
        error::Error,
        io::{Reader, Writer},
    },
};

#[cfg(not(feature = "alloc"))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub struct Segment<'a> {
    pub apdu_type: ApduType,
    pub more_follows: bool,

    pub invoke_id: u8,
    pub sequence_number: u8,
    pub window_size: u8, // number of segments before an Segment-ACK must be sent
    pub service_choice: u8,

    // apdu data
    pub data: &'a [u8],
}

#[cfg(feature = "alloc")]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Clone)]
pub struct Segment<'a> {
    pub apdu_type: ApduType,
    pub more_follows: bool,

    pub invoke_id: u8,
    pub sequence_number: u8,
    pub window_size: u8, // number of segments before an Segment-ACK must be sent
    pub service_choice: u8,

    // apdu data
    pub data: Vec<u8>,
    _phantom: &'a Phantom,
}

impl<'a> Segment<'a> {
    #[cfg(feature = "alloc")]
    pub fn new(
        apdu_type: ApduType,
        more_follows: bool,
        invoke_id: u8,
        sequence_number: u8,
        window_size: u8,
        service_choice: u8,
        data: Vec<u8>,
    ) -> Self {
        use crate::common::spooky::PHANTOM;

        Segment {
            apdu_type,
            more_follows,
            invoke_id,
            sequence_number,
            window_size,
            service_choice,
            data,
            _phantom: &PHANTOM,
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn new(
        apdu_type: ApduType,
        more_follows: bool,
        invoke_id: u8,
        sequence_number: u8,
        window_size: u8,
        service_choice: u8,
        data: &'a [u8],
    ) -> Self {
        Segment {
            apdu_type,
            more_follows,
            invoke_id,
            sequence_number,
            window_size,
            service_choice,
            data,
        }
    }

    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(
        more_follows: bool,
        apdu_type: ApduType,
        reader: &mut Reader,
        buf: &'a [u8],
    ) -> Result<Self, Error> {
        let invoke_id = reader.read_byte(buf)?;
        let sequence_number = reader.read_byte(buf)?;
        let window_size = reader.read_byte(buf)?;
        let service_choice = reader.read_byte(buf)?;
        let data = Self::decode_data(reader, buf)?;

        let segment = Self::new(
            apdu_type,
            more_follows,
            invoke_id,
            sequence_number,
            window_size,
            service_choice,
            data,
        );
        Ok(segment)
    }

    #[cfg(feature = "alloc")]
    fn decode_data(reader: &mut Reader, buf: &[u8]) -> Result<Vec<u8>, Error> {
        // read bytes into owned datastructure
        let len = reader.end - reader.index;
        let mut data = vec![0u8; len];
        let slice = reader.read_slice(len, buf)?;
        data.copy_from_slice(slice);
        Ok(data)
    }

    #[cfg(not(feature = "alloc"))]
    fn decode_data(reader: &mut Reader, buf: &'a [u8]) -> Result<&'a [u8], Error> {
        // read bytes into owned datastructure
        let len = reader.end - reader.index;
        let data = reader.read_slice(len, buf)?;
        Ok(data)
    }

    pub fn encode(&self, writer: &mut Writer) {
        let mut control = ((self.apdu_type.clone() as u8) << 4) | PduFlags::SegmentedMessage as u8;
        if self.more_follows {
            control |= PduFlags::MoreFollows as u8;
        }
        writer.push(control);
        writer.push(self.invoke_id);
        writer.push(self.sequence_number);
        writer.push(self.window_size);
        writer.push(self.service_choice);
        writer.extend_from_slice(&self.data);
    }

    // a special case encoder for when this segment is being accumulated
    // into an unsegmented APDU.
    // returns number of bytes written (TODO: why? - this is redundant and can be calculated by the client)
    pub fn encode_for_accumulation(&self, writer: &mut Writer) -> usize {
        let start = writer.index;
        if self.sequence_number == 0 {
            writer.push((self.apdu_type.clone() as u8) << 4);
            writer.push(self.invoke_id);
            writer.push(self.service_choice);
        }
        writer.extend_from_slice(&self.data);
        writer.index - start
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        application_protocol::application_pdu::ApduType,
        common::io::{Reader, Writer},
    };

    use super::Segment;

    #[test]
    fn reversable() {
        // decoding
        let input: [u8; 7] = [60, 1, 0, 1, 1, 2, 3];
        let mut reader = Reader::new_with_len(input.len() - 1);
        let decoded =
            Segment::decode(true, ApduType::ComplexAck, &mut reader, &input[1..]).unwrap();

        // encoding
        let mut output: [u8; 7] = [0; 7];
        let mut writer = Writer::new(&mut output);
        decoded.encode(&mut writer);
        assert_eq!(input, output);
    }

    #[test]
    fn decoding() {
        // decoding
        let input: [u8; 6] = [1, 12, 1, 1, 2, 3];
        let mut reader = Reader::new_with_len(input.len());
        let decoded = Segment::decode(false, ApduType::ComplexAck, &mut reader, &input).unwrap();
        assert_eq!(decoded.more_follows, false);
        assert_eq!(decoded.sequence_number, 12);
        assert_eq!(decoded.window_size, 1);
        assert_eq!(decoded.apdu_type, ApduType::ComplexAck);
    }
}
