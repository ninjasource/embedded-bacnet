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

impl MstpFrameType {
    #[rustfmt::skip]
    pub fn has_npdu(self) -> bool {
        matches!(self,
           | Self::BacnetDataExpectingReply
           | Self::BacnetDataNotExpectingReply
           | Self::BacnetExtendedDataExpectingReply
           | Self::BacnetExtendedDataNotExpectingReply
        )
    }

    pub fn is_cobs_encoded(self) -> bool {
        matches!(self as u8, 32..=127)
    }
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

/// Preamble at the start of a frame.
const PREAMBLE: [u8; 2] = [0x55, 0xFF];

/// Length of the total header.
const HEADER_LEN: usize = 8;

/// Length of the CRC for non COBS-encoded data.
const DATA_CRC_LEN: usize = 2;

/// Length of the CRC32K after COBS encoding.
const COBS_ENCODED_CRC_LEN: usize = 5;

/// Offset of the length field in the MS/TP frame header.
const HEADER_LEN_OFFSET: usize = 5;

/// Offset of the header CRC field in the MS/TP frame header.
const HEADER_CRC_OFFSET: usize = 7;

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ScanError {
    Garbage(usize),
    InvalidHeader,
    IncompleteHeader,
}

impl ScanError {
    /// Number of bytes to discard from the front of the input stream.
    pub fn discard_len(&self) -> usize {
        match self {
            Self::Garbage(x) => *x,
            Self::InvalidHeader => PREAMBLE.len(),
            Self::IncompleteHeader => 0,
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

    pub fn encode(&self, writer: &mut Writer) -> Result<(), Error> {
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

        match self.frame_type.is_cobs_encoded() {
            // For old style frames with data, just add the CRC.
            false => {
                if writer.index != data_start {
                    let crc_value = !data_crc(&writer.to_bytes()[data_start..]); // flip the bits of the CRC value
                    writer.extend_from_slice(&crc_value.to_le_bytes()); // little endian!
                }
            }
            // For COBS-encoded frames, encode the payload.
            true => {
                // Perform COBS encoding on the message, including the CRC.
                let message_buffer = &mut writer.buf[data_start..];
                let message_len = writer.index - data_start;
                let encoded_len = encode_cobs(message_buffer, message_len)?;
                // Update the position of the writer.
                writer.index = data_start + encoded_len;

                // Add the CRC32K checksum (5 bytes...)
                let crc_value = encoded_crc32k(&writer.buf[data_start..writer.index]);
                writer.extend_from_slice(&crc_value); // little endian!
            }
        }

        // If there is a payload, update the length field in the header.
        if writer.index != data_start {
            // Exclude the 2 byte checksum from the length, even for COBS encoded frames (for backwards compatibility).
            let len = writer.index - data_start - DATA_CRC_LEN;
            let len = u16::try_from(len)
                .map_err(|_| Error::Length(("message payload too long", len as u32)))?;
            writer.buf[header_start + HEADER_LEN_OFFSET..][..2].copy_from_slice(&len.to_be_bytes());
        }

        let header_crc = !header_crc(&writer.to_bytes()[header_start..data_start]); // NOTE: flip the bits of the CRC
        writer.buf[header_start + HEADER_CRC_OFFSET] = header_crc;
        Ok(())
    }

    /// Scan the reader for an incoming frame.
    ///
    /// If a valid frame header is found at the start of the buffer,
    /// this function returns the length of the whole frame.
    /// Note that this does not guarantee that the complete data payload is also already in the buffer.
    ///
    /// If no valid frame header is present at the start of the buffer,
    /// a [`ScanError`] is returned which will tell you how much data to discard from the buffer.
    ///
    /// Does not modify the position of the reader.
    pub fn scan(reader: &mut Reader, buf: &'a [u8]) -> Result<usize, ScanError> {
        let buf = &buf[reader.index..reader.end];

        // Count bytes before the first valid pre-amble.
        let garbage = buf
            .array_windows::<2>()
            .take_while(|&&data| data != PREAMBLE)
            .count();
        if garbage > 0 {
            return Err(ScanError::Garbage(garbage));
        }

        // Make sure the buffer contains a complete header.
        if buf.len() < HEADER_LEN {
            return Err(ScanError::IncompleteHeader);
        }

        // Check the header CRC.
        if header_crc(&buf[2..8]) != 0x55 {
            return Err(ScanError::InvalidHeader);
        }

        // Report the total frame size.
        let data_len = u16::from_be_bytes([buf[5], buf[6]]) as usize;
        Ok(HEADER_LEN + data_len + DATA_CRC_LEN)
    }

    /// Decode a BACnet MS/TP frame.
    ///
    /// NOTE: This will modify the buffer in-place to decode COBS-encoded payloads.
    #[cfg_attr(feature = "alloc", bacnet_macros::remove_lifetimes_from_fn_args)]
    pub fn decode(reader: &mut Reader, buf: &'a mut [u8]) -> Result<Self, Error> {
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

        // Get a mutable slice for the payload data with (encoded) CRC.
        let data_start = reader.index;
        reader.read_slice(data_len + DATA_CRC_LEN, buf)?;
        let data = &mut buf[data_start..][..data_len + DATA_CRC_LEN];

        // For "simple" frames, just check the data CRC and then remove it.
        let data = match frame_type.is_cobs_encoded() {
            false => {
                if data_crc(data) != 0xF0B8 {
                    return Err(Error::InvalidValue("invalid MS/TP data CRC"));
                }
                &data[..data_len]
            }
            // For COBS encoded frames, things are a bit more complicated.
            true => {
                if data.len() < COBS_ENCODED_CRC_LEN {
                    return Err(Error::Length((
                        "COBS encoded payload is too short",
                        data.len() as u32,
                    )));
                }

                // First decode the CRC.
                // This leaves the decoded CRC in the 4 bytes after the COBS encoded data.
                let crc_start = data.len() - COBS_ENCODED_CRC_LEN;
                decode_cobs(&mut data[crc_start..])
                    .map_err(|()| Error::ConvertDataLink("invalid COBS encoded CRC"))?;

                // Now verify the CRC.
                if crc32k(&data[..crc_start + 4]) != 0x0843323B {
                    return Err(Error::ConvertDataLink(
                        "Invalid MS/TP COBS-encoded data CRC",
                    ));
                }

                // Now decode the COBS data itself.
                let decoded_len = decode_cobs(&mut data[..crc_start])
                    .map_err(|()| Error::ConvertDataLink("invalid COBS encoded data payload"))?;
                &data[..decoded_len]
            }
        };

        let npdu = match frame_type.has_npdu() {
            false => None,
            true => Some(NetworkPdu::decode(
                &mut Reader::new_with_len(data.len()),
                data,
            )?),
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

fn crc32k(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF;

    for &byte in data {
        // NOTE: This CRC works the opposite way from normal because ASHRAE decided that bit 7 represents x^0 and bit 0 represents x^7.
        // For this reason, we XOR each data byte with the least significant byte of the CRC accumulator instead of the most significant byte.
        let index = (crc & 0xFF) as usize ^ usize::from(byte);
        crc = crc >> 8 ^ CRC32K_TABLE[index]
    }

    // NOTE: When encoded in the frame, the bits must be flipped, but the CRC value itself is not bit-flipped.
    // This matters because the specification also tells you the CRC value to check for, and when they do it is *without* flipped bits.
    crc
}

fn encoded_crc32k(data: &[u8]) -> [u8; 5] {
    let crc = crc32k(data);
    let crc = !crc; // flip bits
    let crc = crc.to_le_bytes(); // little endian
    let mut encoded = [0u8; 6];
    let len = corncobs::encode_buf(&crc, &mut encoded);
    debug_assert_eq!(len, 6);
    [encoded[0], encoded[1], encoded[2], encoded[3], encoded[4]]
}

fn decode_cobs(data: &mut [u8]) -> Result<usize, ()> {
    // XOR all bytes with 0x55 before decoding.
    for byte in data.iter_mut() {
        *byte ^= 0x55;
    }
    corncobs::decode_in_place(data).map_err(|_| ())
}

fn encode_cobs(buffer: &mut [u8], message_len: usize) -> Result<usize, Error> {
    if buffer.len() < corncobs::max_encoded_len(message_len) {
        return Err(Error::Length((
            "buffer not large enough to encode COBS payload",
            buffer.len() as u32,
        )));
    }

    let encoded_len = corncobs::encode_in_place(buffer, message_len);
    let encoded_len = encoded_len - 1; // COBS adds a trailing 0 byte, BACnet does not

    // XOR all bytes with 0x55 after endecoding.
    for byte in &mut buffer[..encoded_len] {
        *byte ^= 0x55;
    }

    Ok(encoded_len)
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

/// CRC table for CRC32K (with most significant bit representing x^0)
#[rustfmt::skip]
const CRC32K_TABLE: [u32; 256] =  [
    0x00000000, 0x9695C4CA, 0xFB4839C9, 0x6DDDFD03, 0x20F3C3CF, 0xB6660705, 0xDBBBFA06, 0x4D2E3ECC,
    0x41E7879E, 0xD7724354, 0xBAAFBE57, 0x2C3A7A9D, 0x61144451, 0xF781809B, 0x9A5C7D98, 0x0CC9B952,
    0x83CF0F3C, 0x155ACBF6, 0x788736F5, 0xEE12F23F, 0xA33CCCF3, 0x35A90839, 0x5874F53A, 0xCEE131F0,
    0xC22888A2, 0x54BD4C68, 0x3960B16B, 0xAFF575A1, 0xE2DB4B6D, 0x744E8FA7, 0x199372A4, 0x8F06B66E,
    0xD1FDAE25, 0x47686AEF, 0x2AB597EC, 0xBC205326, 0xF10E6DEA, 0x679BA920, 0x0A465423, 0x9CD390E9,
    0x901A29BB, 0x068FED71, 0x6B521072, 0xFDC7D4B8, 0xB0E9EA74, 0x267C2EBE, 0x4BA1D3BD, 0xDD341777,
    0x5232A119, 0xC4A765D3, 0xA97A98D0, 0x3FEF5C1A, 0x72C162D6, 0xE454A61C, 0x89895B1F, 0x1F1C9FD5,
    0x13D52687, 0x8540E24D, 0xE89D1F4E, 0x7E08DB84, 0x3326E548, 0xA5B32182, 0xC86EDC81, 0x5EFB184B,
    0x7598EC17, 0xE30D28DD, 0x8ED0D5DE, 0x18451114, 0x556B2FD8, 0xC3FEEB12, 0xAE231611, 0x38B6D2DB,
    0x347F6B89, 0xA2EAAF43, 0xCF375240, 0x59A2968A, 0x148CA846, 0x82196C8C, 0xEFC4918F, 0x79515545,
    0xF657E32B, 0x60C227E1, 0x0D1FDAE2, 0x9B8A1E28, 0xD6A420E4, 0x4031E42E, 0x2DEC192D, 0xBB79DDE7,
    0xB7B064B5, 0x2125A07F, 0x4CF85D7C, 0xDA6D99B6, 0x9743A77A, 0x01D663B0, 0x6C0B9EB3, 0xFA9E5A79,
    0xA4654232, 0x32F086F8, 0x5F2D7BFB, 0xC9B8BF31, 0x849681FD, 0x12034537, 0x7FDEB834, 0xE94B7CFE,
    0xE582C5AC, 0x73170166, 0x1ECAFC65, 0x885F38AF, 0xC5710663, 0x53E4C2A9, 0x3E393FAA, 0xA8ACFB60,
    0x27AA4D0E, 0xB13F89C4, 0xDCE274C7, 0x4A77B00D, 0x07598EC1, 0x91CC4A0B, 0xFC11B708, 0x6A8473C2,
    0x664DCA90, 0xF0D80E5A, 0x9D05F359, 0x0B903793, 0x46BE095F, 0xD02BCD95, 0xBDF63096, 0x2B63F45C,
    0xEB31D82E, 0x7DA41CE4, 0x1079E1E7, 0x86EC252D, 0xCBC21BE1, 0x5D57DF2B, 0x308A2228, 0xA61FE6E2,
    0xAAD65FB0, 0x3C439B7A, 0x519E6679, 0xC70BA2B3, 0x8A259C7F, 0x1CB058B5, 0x716DA5B6, 0xE7F8617C,
    0x68FED712, 0xFE6B13D8, 0x93B6EEDB, 0x05232A11, 0x480D14DD, 0xDE98D017, 0xB3452D14, 0x25D0E9DE,
    0x2919508C, 0xBF8C9446, 0xD2516945, 0x44C4AD8F, 0x09EA9343, 0x9F7F5789, 0xF2A2AA8A, 0x64376E40,
    0x3ACC760B, 0xAC59B2C1, 0xC1844FC2, 0x57118B08, 0x1A3FB5C4, 0x8CAA710E, 0xE1778C0D, 0x77E248C7,
    0x7B2BF195, 0xEDBE355F, 0x8063C85C, 0x16F60C96, 0x5BD8325A, 0xCD4DF690, 0xA0900B93, 0x3605CF59,
    0xB9037937, 0x2F96BDFD, 0x424B40FE, 0xD4DE8434, 0x99F0BAF8, 0x0F657E32, 0x62B88331, 0xF42D47FB,
    0xF8E4FEA9, 0x6E713A63, 0x03ACC760, 0x953903AA, 0xD8173D66, 0x4E82F9AC, 0x235F04AF, 0xB5CAC065,
    0x9EA93439, 0x083CF0F3, 0x65E10DF0, 0xF374C93A, 0xBE5AF7F6, 0x28CF333C, 0x4512CE3F, 0xD3870AF5,
    0xDF4EB3A7, 0x49DB776D, 0x24068A6E, 0xB2934EA4, 0xFFBD7068, 0x6928B4A2, 0x04F549A1, 0x92608D6B,
    0x1D663B05, 0x8BF3FFCF, 0xE62E02CC, 0x70BBC606, 0x3D95F8CA, 0xAB003C00, 0xC6DDC103, 0x504805C9,
    0x5C81BC9B, 0xCA147851, 0xA7C98552, 0x315C4198, 0x7C727F54, 0xEAE7BB9E, 0x873A469D, 0x11AF8257,
    0x4F549A1C, 0xD9C15ED6, 0xB41CA3D5, 0x2289671F, 0x6FA759D3, 0xF9329D19, 0x94EF601A, 0x027AA4D0,
    0x0EB31D82, 0x9826D948, 0xF5FB244B, 0x636EE081, 0x2E40DE4D, 0xB8D51A87, 0xD508E784, 0x439D234E,
    0xCC9B9520, 0x5A0E51EA, 0x37D3ACE9, 0xA1466823, 0xEC6856EF, 0x7AFD9225, 0x17206F26, 0x81B5ABEC,
    0x8D7C12BE, 0x1BE9D674, 0x76342B77, 0xE0A1EFBD, 0xAD8FD171, 0x3B1A15BB, 0x56C7E8B8, 0xC0522C72,
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
        extern crate std;
        std::eprint!("const HEADER_CRC_TABLE: [u8; 256] =  [");
        for i in 0..=255 {
            if i % 16 == 0 {
                std::eprint!("\n   ");
            }
            std::eprint!(" 0x{:02X},", crc_remainder(i));
        }
        std::eprintln!("\n];");

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

        extern crate std;
        std::eprint!("const DATA_CRC_TABLE: [u16; 256] =  [");
        for i in 0..=255 {
            if i % 16 == 0 {
                std::eprint!("\n   ");
            }
            std::eprint!(" 0x{:04X},", crc_remainder(i));
        }
        std::eprintln!("\n];");

        for i in 0..=255 {
            assert!(DATA_CRC_TABLE[usize::from(i)] == crc_remainder(i));
        }
    }

    #[test]
    fn crc32k_table() {
        const fn crc_remainder(data: u8) -> u32 {
            const POLYNOMIAL: u32 = 0xEB31D82E;
            let mut data = data as u32;
            let mut i = 0;
            while i < 8 {
                i += 1;
                if data & 0x01 != 0 {
                    data = (data >> 1) ^ POLYNOMIAL;
                } else {
                    data >>= 1;
                }
            }
            data
        }

        extern crate std;
        std::eprint!("const CRC32K_TABLE: [u32; 256] =  [");
        for i in 0..=255 {
            if i % 8 == 0 {
                std::eprint!("\n   ");
            }
            std::eprint!(" 0x{:08X},", crc_remainder(i));
        }
        std::eprintln!("\n];");

        for i in 0..=255 {
            assert!(CRC32K_TABLE[usize::from(i)] == crc_remainder(i));
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

    #[test]
    fn test_crc32k() {
        // Example from the specification.
        assert_eq!(crc32k(&[0x01, 0x22, 0x30]), 0x83DD5A41);
        assert_eq!(
            crc32k(&[0x01, 0x22, 0x30, 0xBE, 0xA5, 0x22, 0x7C]),
            0x0843323B
        );
    }

    #[test]
    fn test_mstp_who_has_decode() {
        use crate::application_protocol::unconfirmed::UnconfirmedServiceChoice::WhoHas;
        use crate::common::error::Unimplemented::UnconfirmedServiceChoice;

        // Example from the specification.
        #[rustfmt::skip]
        let mut data = [
            0x55, 0xFF, 0x21, 0xFF, 0x01, 0x02, 0x00, 0x4E, 0x50, 0x54, 0x75, 0xAA, 0xAA, 0x5D, 0xAA, 0x45,
            0x52, 0x68, 0xAB, 0x54, 0xBA, 0xAA, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14,
            0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17,
            0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x17, 0x16, 0x16, 0x16, 0x16,
            0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x16, 0x11,
            0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
            0x11, 0x11, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
            0x10, 0x10, 0x10, 0x10, 0x10, 0x13, 0x13, 0x13, 0x13, 0x13, 0x13, 0x13, 0x13, 0x13, 0x13, 0x13,
            0x13, 0x13, 0x13, 0x13, 0x13, 0x13, 0x13, 0x13, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12,
            0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D,
            0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1D, 0x1C, 0x1C,
            0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C, 0x1C,
            0x1C, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F, 0x1F,
            0x1F, 0x1F, 0x1F, 0x1F, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E,
            0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x1E, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19,
            0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x19, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18,
            0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x1B, 0x1B, 0x1B,
            0x1B, 0x1B, 0x1B, 0x1B, 0xA4, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B, 0x1B,
            0x1B, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A, 0x1A,
            0x1A, 0x1A, 0x1A, 0x1A, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05,
            0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x05, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
            0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
            0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x06, 0x06, 0x06,
            0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06, 0x06,
            0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03,
            0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
            0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x0D, 0x0D, 0x0D, 0x0D,
            0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0D, 0x0C,
            0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C,
            0x0C, 0x0C, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x0F,
            0x0F, 0x0F, 0x0F, 0x0F, 0x0F, 0x50, 0xF9, 0xA1, 0xD6, 0xE8,
        ];

        let mut reader = crate::common::io::Reader::new_with_len(data.len());

        // Sadly,this is a WhoHas service, which is not supported, so for now we test that we get the correct `Unimplemented` error.
        let result = MstpFrame::decode(&mut reader, &mut data);
        assert!(matches!(
            result,
            Err(Error::Unimplemented(UnconfirmedServiceChoice(WhoHas)))
        ));
    }
}
