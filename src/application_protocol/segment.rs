use core::usize;

use crate::common::{error::Error, io::{Reader, Writer}};

use super::application_pdu::{ApduType, PduFlags};

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

impl<'a> Segment<'a> {
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

        Ok(Segment {
            apdu_type,
            more_follows,

            invoke_id,
            sequence_number,
            window_size,
            service_choice,

            data: &buf[reader.index..],
        })
    }

    pub fn encode(&self, writer: &mut Writer) {
        let mut control = ((self.apdu_type.clone() as u8) << 4) | PduFlags::SegmentedMessage as u8;
        if self.more_follows {
            control = control | PduFlags::MoreFollows as u8;
        }
        writer.push(control);
        writer.push(self.invoke_id);
        writer.push(self.sequence_number);
        writer.push(self.window_size);
        writer.push(self.service_choice);
        writer.extend_from_slice(self.data);
    }

    // a special case encoder for when this segment is being accumulated
    // into an unsegmented APDU.
    pub fn encode_for_accumulation(&self, writer: &mut Writer) -> usize {
        let mut written = 0;
        if self.sequence_number == 0 {
            writer.push((self.apdu_type.clone() as u8) << 4);
            writer.push(self.invoke_id);
            writer.push(self.service_choice);
            written += 3;
        }
        writer.extend_from_slice(self.data);
        written += self.data.len();
        
        written
    }
}

#[cfg(test)]
mod tests {
    use crate::{application_protocol::application_pdu::ApduType, common::io::{Reader, Writer}};

    use super::Segment;
 
    #[test]
    fn reversable() {
        // decoding
        let input: [u8; 7] = [60,1,0,1,1,2,3];
        let mut reader = Reader::new_with_len(input.len() - 1);
        let decoded = Segment::decode(true, ApduType::ComplexAck, &mut reader, &input[1..]).unwrap();
       
        // encoding
        let mut output: [u8; 7] = [0; 7];
        let mut writer = Writer::new(&mut output);
        decoded.encode(&mut writer);
        assert_eq!(input, output);
    }
    
    #[test]
    fn decoding() {
        // decoding
        let input: [u8; 6] = [1,12,1,1,2,3];
        let mut reader = Reader::new_with_len(input.len());
        let decoded = Segment::decode(false, ApduType::ComplexAck, &mut reader, &input).unwrap(); 
        assert_eq!(decoded.more_follows, false);
        assert_eq!(decoded.sequence_number, 12);
        assert_eq!(decoded.window_size, 1);
        assert_eq!(decoded.apdu_type, ApduType::ComplexAck);
    }
}
