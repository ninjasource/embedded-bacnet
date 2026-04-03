use crate::{
    application_protocol::{application_pdu::ApplicationPdu, confirmed::ConfirmedRequest},
    common::{
        error::Error,
        io::{Reader, Writer},
    },
    network_protocol::network_pdu::{MessagePriority, NetworkMessage, NetworkPdu},
};

// Bacnet Virtual Link Control
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MstpFrame<'a> {
    pub frame_type: MstpFrameType,
    pub destination_address: u8,
    pub source_address: u8,
    pub npdu: Option<NetworkPdu<'a>>,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum MstpFrameType {
    Token = 0,
    PollForManager = 1,
    ReplyToPollForManager = 2,
    TestRequest = 3,
    TestResponse = 4,
    BacnetDataExpectingReply = 5,
    BacnetDataNotExpectingReply = 6,
    ReplyPostponed = 7,
    BacnetExtendedDataExpectingReply = 32,
    BacnetExtendedDataNotExpectingReply = 33,
}

impl From<MstpFrameType> for u8 {
    fn from(value: MstpFrameType) -> Self {
        value as Self
    }
}

impl TryFrom<u8> for MstpFrameType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Token),
            1 => Ok(Self::PollForManager),
            2 => Ok(Self::ReplyToPollForManager),
            3 => Ok(Self::TestRequest),
            4 => Ok(Self::TestResponse),
            5 => Ok(Self::BacnetDataExpectingReply),
            6 => Ok(Self::BacnetDataNotExpectingReply),
            7 => Ok(Self::ReplyPostponed),
            32 => Ok(Self::BacnetExtendedDataExpectingReply),
            33 => Ok(Self::BacnetExtendedDataNotExpectingReply),
            x => Err(x),
        }
    }
}

const PREAMBLE: [u8; 2] = [0x55, 0xFF];

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ScanError {
    Garbage(usize),
    InvalidHeader,
    IncompleteFrame,
}

impl ScanError {
    /// Number of bytes to discard from the front of the input stream.
    pub fn discard_len(&self) -> usize {
        match self {
            Self::Garbage(x) => *x,
            Self::InvalidHeader => 2,
            Self::IncompleteFrame => 0,
        }
    }
}

impl<'a> MstpFrame<'a> {
    pub fn new(
        frame_type: MstpFrameType,
        destination_address: u8,
        source_address: u8,
        npdu: Option<NetworkPdu<'a>>,
    ) -> Self {
        Self {
            frame_type,
            destination_address,
            source_address,
            npdu,
        }
    }

    pub fn new_confirmed_req(
        destination_address: u8,
        source_address: u8,
        req: ConfirmedRequest<'a>,
    ) -> Self {
        let apdu = ApplicationPdu::ConfirmedRequest(req);
        let message = NetworkMessage::Apdu(apdu);
        let npdu = NetworkPdu::new(None, None, true, MessagePriority::Normal, message);
        Self::new(
            MstpFrameType::BacnetDataExpectingReply,
            destination_address,
            source_address,
            Some(npdu),
        )
    }

    pub fn encode(&self, writer: &mut Writer) {
        let header_start = writer.index;
        writer.extend_from_slice(&PREAMBLE);
        writer.extend_from_slice(&[
            self.frame_type.into(),
            self.destination_address,
            self.source_address,
            0, // length placeholder (2 bytes)
            0,
            0, // header CRC placeholder
        ]);
        let data_start = writer.index;
        if let Some(npdu) = &self.npdu {
            npdu.encode(writer);
        }
        let len = u16::try_from(writer.index - data_start).unwrap(); // TODO: encoding should probably be fallible too.
        if len > 0 {
            writer.buf[header_start + 5..][..2].copy_from_slice(&len.to_be_bytes());
            let data_crc = !data_crc(&writer.to_bytes()[data_start..]); // NOTE: flip the bits of the CRC
            writer.extend_from_slice(&data_crc.to_le_bytes()); // little endian!
        }

        let header_crc = !header_crc(&writer.to_bytes()[header_start..data_start]); // NOTE: flip the bits of the CRC
        writer.buf[header_start + 7] = header_crc;
    }

    /// Scan the reader for an incoming frame.
    ///
    /// If a valid frame header is found in the buffer, this returns the length of the whole frame.
    /// Otherwise, a [`ScanError`] is returned which can tell you how much data to discard from the buffer.
    ///
    /// Does not modify the position of the reader.
    pub fn scan(reader: &mut Reader, buf: &'a [u8]) -> Result<usize, ScanError> {
        let buf = &buf[reader.index..reader.end];
        let garbage = buf
            .array_windows::<2>()
            .take_while(|&&data| data != PREAMBLE)
            .count();
        if garbage > 0 {
            return Err(ScanError::Garbage(garbage));
        }

        if buf.len() < 8 {
            return Err(ScanError::IncompleteFrame);
        }

        // Check the header CRC (before checking length, it's a better error for corrupt transmissions).
        if header_crc(&buf[2..8]) != 0x55 {
            return Err(ScanError::InvalidHeader);
        }

        let data_len = u16::from_be_bytes([buf[5], buf[6]]) as usize;
        Ok(data_len + 8)
    }

    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(reader: &mut Reader, buf: &'a [u8]) -> Result<Self, Error> {
        let preamble: [u8; 2] = reader.read_bytes(buf)?;
        if preamble != PREAMBLE {
            return Err(Error::InvalidValue("invalid MS/TP frame preamble"));
        }
        let header_start = reader.index;

        let frame_type = MstpFrameType::try_from(reader.read_byte(buf)?)
            .map_err(|_| Error::InvalidValue("invalid or unrecognized MS/TP frame type"))?;
        let destination_address = reader.read_byte(buf)?;
        let source_address = reader.read_byte(buf)?;
        let data_len = u16::from_be_bytes(reader.read_bytes(buf)?) as usize;
        let _header_crc = reader.read_byte(buf)?;
        let data_start = reader.index;

        // Check the header CRC (before checking length, it's a better error for corrupt transmissions).
        if header_crc(&buf[header_start..data_start]) != 0x55 {
            return Err(Error::InvalidValue("invalid MS/TP header CRC"));
        }

        if data_len == 0 {
            return Ok(Self {
                frame_type,
                destination_address,
                source_address,
                npdu: None,
            });
        }

        // Read the data and check the CRC.
        let data_with_crc = reader.read_slice(data_len + 2, buf)?; // Plus 2 bytes for the CRC (even correct extended data frames).
        if data_crc(data_with_crc) != 0xF0B8 {
            return Err(Error::InvalidValue("invalid MS/TP data CRC"));
        }

        let npdu = match frame_type {
            MstpFrameType::BacnetDataExpectingReply
            | MstpFrameType::BacnetExtendedDataNotExpectingReply => Some(NetworkPdu::decode(
                &mut Reader::new_with_len(data_len),
                data_with_crc,
            )?),
            _ => None,
        };

        Ok(Self {
            frame_type,
            destination_address,
            source_address,
            npdu,
        })
    }
}

fn header_crc(data: &[u8]) -> u8 {
    let mut crc = 0xFF;
    for byte in data {
        crc = HEADER_CRC_TABLE[usize::from(crc ^ byte)];
    }

    // NOTE: When encoded in the frame, the bits must be flipped, but the CRC value itself is not bit-flipped.
    // This matters because the specification also tells you the CRC value to check for, and when they do it is *without* flipped bits.
    crc
}

fn data_crc(data: &[u8]) -> u16 {
    let mut crc = 0xFFFF;

    for &byte in data {
        // NOTE: This CRC works the opposite way from normal because ASHRAE decided that bit 7 represents x^0 and bit 0 represents x^7.
        // For this reason, we XOR each data byte with the least significant byte of the CRC accumulator instead of the most significant byte.
        let index = usize::from(crc) & 0xFF ^ usize::from(byte);
        crc = crc >> 8 ^ DATA_CRC_TABLE[index]
    }

    // NOTE: When encoded in the frame, the bits must be flipped, but the CRC value itself is not bit-flipped.
    // This matters because the specification also tells you the CRC value to check for, and when they do it is *without* flipped bits.
    crc
}

/// CRC table for the header checksum.
#[rustfmt::skip]
const HEADER_CRC_TABLE: [u8; 256] = [
    0x00, 0xFE, 0xFF, 0x01, 0xFD, 0x03, 0x02, 0xFC,
    0xF9, 0x07, 0x06, 0xF8, 0x04, 0xFA, 0xFB, 0x05,
    0xF1, 0x0F, 0x0E, 0xF0, 0x0C, 0xF2, 0xF3, 0x0D,
    0x08, 0xF6, 0xF7, 0x09, 0xF5, 0x0B, 0x0A, 0xF4,
    0xE1, 0x1F, 0x1E, 0xE0, 0x1C, 0xE2, 0xE3, 0x1D,
    0x18, 0xE6, 0xE7, 0x19, 0xE5, 0x1B, 0x1A, 0xE4,
    0x10, 0xEE, 0xEF, 0x11, 0xED, 0x13, 0x12, 0xEC,
    0xE9, 0x17, 0x16, 0xE8, 0x14, 0xEA, 0xEB, 0x15,
    0xC1, 0x3F, 0x3E, 0xC0, 0x3C, 0xC2, 0xC3, 0x3D,
    0x38, 0xC6, 0xC7, 0x39, 0xC5, 0x3B, 0x3A, 0xC4,
    0x30, 0xCE, 0xCF, 0x31, 0xCD, 0x33, 0x32, 0xCC,
    0xC9, 0x37, 0x36, 0xC8, 0x34, 0xCA, 0xCB, 0x35,
    0x20, 0xDE, 0xDF, 0x21, 0xDD, 0x23, 0x22, 0xDC,
    0xD9, 0x27, 0x26, 0xD8, 0x24, 0xDA, 0xDB, 0x25,
    0xD1, 0x2F, 0x2E, 0xD0, 0x2C, 0xD2, 0xD3, 0x2D,
    0x28, 0xD6, 0xD7, 0x29, 0xD5, 0x2B, 0x2A, 0xD4,
    0x81, 0x7F, 0x7E, 0x80, 0x7C, 0x82, 0x83, 0x7D,
    0x78, 0x86, 0x87, 0x79, 0x85, 0x7B, 0x7A, 0x84,
    0x70, 0x8E, 0x8F, 0x71, 0x8D, 0x73, 0x72, 0x8C,
    0x89, 0x77, 0x76, 0x88, 0x74, 0x8A, 0x8B, 0x75,
    0x60, 0x9E, 0x9F, 0x61, 0x9D, 0x63, 0x62, 0x9C,
    0x99, 0x67, 0x66, 0x98, 0x64, 0x9A, 0x9B, 0x65,
    0x91, 0x6F, 0x6E, 0x90, 0x6C, 0x92, 0x93, 0x6D,
    0x68, 0x96, 0x97, 0x69, 0x95, 0x6B, 0x6A, 0x94,
    0x40, 0xBE, 0xBF, 0x41, 0xBD, 0x43, 0x42, 0xBC,
    0xB9, 0x47, 0x46, 0xB8, 0x44, 0xBA, 0xBB, 0x45,
    0xB1, 0x4F, 0x4E, 0xB0, 0x4C, 0xB2, 0xB3, 0x4D,
    0x48, 0xB6, 0xB7, 0x49, 0xB5, 0x4B, 0x4A, 0xB4,
    0xA1, 0x5F, 0x5E, 0xA0, 0x5C, 0xA2, 0xA3, 0x5D,
    0x58, 0xA6, 0xA7, 0x59, 0xA5, 0x5B, 0x5A, 0xA4,
    0x50, 0xAE, 0xAF, 0x51, 0xAD, 0x53, 0x52, 0xAC,
    0xA9, 0x57, 0x56, 0xA8, 0x54, 0xAA, 0xAB, 0x55,
];

/// CRC table for the data checksum.
#[rustfmt::skip]
const DATA_CRC_TABLE: [u16; 256] = [
    0x0000, 0x1189, 0x2312, 0x329B, 0x4624, 0x57AD, 0x6536, 0x74BF,
    0x8C48, 0x9DC1, 0xAF5A, 0xBED3, 0xCA6C, 0xDBE5, 0xE97E, 0xF8F7,
    0x1081, 0x0108, 0x3393, 0x221A, 0x56A5, 0x472C, 0x75B7, 0x643E,
    0x9CC9, 0x8D40, 0xBFDB, 0xAE52, 0xDAED, 0xCB64, 0xF9FF, 0xE876,
    0x2102, 0x308B, 0x0210, 0x1399, 0x6726, 0x76AF, 0x4434, 0x55BD,
    0xAD4A, 0xBCC3, 0x8E58, 0x9FD1, 0xEB6E, 0xFAE7, 0xC87C, 0xD9F5,
    0x3183, 0x200A, 0x1291, 0x0318, 0x77A7, 0x662E, 0x54B5, 0x453C,
    0xBDCB, 0xAC42, 0x9ED9, 0x8F50, 0xFBEF, 0xEA66, 0xD8FD, 0xC974,
    0x4204, 0x538D, 0x6116, 0x709F, 0x0420, 0x15A9, 0x2732, 0x36BB,
    0xCE4C, 0xDFC5, 0xED5E, 0xFCD7, 0x8868, 0x99E1, 0xAB7A, 0xBAF3,
    0x5285, 0x430C, 0x7197, 0x601E, 0x14A1, 0x0528, 0x37B3, 0x263A,
    0xDECD, 0xCF44, 0xFDDF, 0xEC56, 0x98E9, 0x8960, 0xBBFB, 0xAA72,
    0x6306, 0x728F, 0x4014, 0x519D, 0x2522, 0x34AB, 0x0630, 0x17B9,
    0xEF4E, 0xFEC7, 0xCC5C, 0xDDD5, 0xA96A, 0xB8E3, 0x8A78, 0x9BF1,
    0x7387, 0x620E, 0x5095, 0x411C, 0x35A3, 0x242A, 0x16B1, 0x0738,
    0xFFCF, 0xEE46, 0xDCDD, 0xCD54, 0xB9EB, 0xA862, 0x9AF9, 0x8B70,
    0x8408, 0x9581, 0xA71A, 0xB693, 0xC22C, 0xD3A5, 0xE13E, 0xF0B7,
    0x0840, 0x19C9, 0x2B52, 0x3ADB, 0x4E64, 0x5FED, 0x6D76, 0x7CFF,
    0x9489, 0x8500, 0xB79B, 0xA612, 0xD2AD, 0xC324, 0xF1BF, 0xE036,
    0x18C1, 0x0948, 0x3BD3, 0x2A5A, 0x5EE5, 0x4F6C, 0x7DF7, 0x6C7E,
    0xA50A, 0xB483, 0x8618, 0x9791, 0xE32E, 0xF2A7, 0xC03C, 0xD1B5,
    0x2942, 0x38CB, 0x0A50, 0x1BD9, 0x6F66, 0x7EEF, 0x4C74, 0x5DFD,
    0xB58B, 0xA402, 0x9699, 0x8710, 0xF3AF, 0xE226, 0xD0BD, 0xC134,
    0x39C3, 0x284A, 0x1AD1, 0x0B58, 0x7FE7, 0x6E6E, 0x5CF5, 0x4D7C,
    0xC60C, 0xD785, 0xE51E, 0xF497, 0x8028, 0x91A1, 0xA33A, 0xB2B3,
    0x4A44, 0x5BCD, 0x6956, 0x78DF, 0x0C60, 0x1DE9, 0x2F72, 0x3EFB,
    0xD68D, 0xC704, 0xF59F, 0xE416, 0x90A9, 0x8120, 0xB3BB, 0xA232,
    0x5AC5, 0x4B4C, 0x79D7, 0x685E, 0x1CE1, 0x0D68, 0x3FF3, 0x2E7A,
    0xE70E, 0xF687, 0xC41C, 0xD595, 0xA12A, 0xB0A3, 0x8238, 0x93B1,
    0x6B46, 0x7ACF, 0x4854, 0x59DD, 0x2D62, 0x3CEB, 0x0E70, 0x1FF9,
    0xF78F, 0xE606, 0xD49D, 0xC514, 0xB1AB, 0xA022, 0x92B9, 0x8330,
    0x7BC7, 0x6A4E, 0x58D5, 0x495C, 0x3DE3, 0x2C6A, 0x1EF1, 0x0F78,
];

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header_crc_table() {
        fn crc_remainder(data: u8) -> u8 {
            // NOTE: This CRC works the opposite way from normal because ASHRAE decided that bit 7 represents x^0 and bit 0 represents x^7.
            // Hence, we reverse the bits of the polynomial and we shift right instead of left.
            // For this particular polynomial, flipping the bits is a no-op.
            // Still, leave it here to understand what is happening better.
            const POLYNOMIAL: u8 = 0x81_u8.reverse_bits();
            let mut data = data;
            for _ in 0..8 {
                if data & 0x01 != 0 {
                    data = (data >> 1) ^ POLYNOMIAL;
                } else {
                    data >>= 1;
                }
            }
            data
        }

        for i in 0..=255 {
            assert!(HEADER_CRC_TABLE[usize::from(i)] == crc_remainder(i));
        }
    }

    #[test]
    fn data_crc_table() {
        fn crc_remainder(data: u8) -> u16 {
            // NOTE: This CRC works the opposite way from normal because ASHRAE decided that bit 7 represents x^0 and bit 0 represents x^7.
            // Hence, we reverse the bits of the polynomial and we shift right instead of left.
            const POLYNOMIAL: u16 = 0x1021_u16.reverse_bits();
            let mut crc = u16::from(data);
            for _ in 0..8 {
                if crc & 0x0001 != 0 {
                    crc = (crc >> 1) ^ POLYNOMIAL;
                } else {
                    crc >>= 1;
                }
            }
            crc
        }

        for i in 0..=255 {
            assert!(DATA_CRC_TABLE[usize::from(i)] == crc_remainder(i));
        }
    }

    #[test]
    fn test_header_crc() {
        // Example from the specification.
        assert_eq!(header_crc(&[0x00, 0x10, 0x05, 0x00, 0x00]), 0x73);
        assert_eq!(header_crc(&[0x00, 0x10, 0x05, 0x00, 0x00, 0x8C]), 0x55);
    }

    #[test]
    fn test_data_crc() {
        // Example from the specification.
        assert_eq!(data_crc(&[0x01, 0x22, 0x30]), 0x42EF);
        assert_eq!(data_crc(&[0x01, 0x22, 0x30, 0x10, 0xBD]), 0xF0B8);
    }
}
