use arrayref::array_ref;
use heapless::Vec;

use super::error::Error;

pub struct Buffer {
    pub buf: Vec<u8, 1024>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn push(&mut self, item: u8) {
        self.buf.push(item).unwrap()
    }

    pub fn extend_from_slice(&mut self, src: &[u8]) {
        self.buf.extend_from_slice(src).unwrap()
    }

    pub fn to_bytes<'a>(&'a self) -> &'a [u8] {
        &self.buf
    }
}

pub struct Reader {
    buf: Vec<u8, 1024>,
    index: usize,
}

impl Reader {
    pub fn new(payload: &[u8]) -> Self {
        let mut buf: Vec<u8, 1024> = Vec::new();
        buf.extend_from_slice(payload).unwrap();
        Self { buf, index: 0 }
    }

    pub fn read_byte(&mut self) -> u8 {
        let byte = self.buf[self.index];
        self.index += 1;
        byte
    }

    pub fn read_bytes<const COUNT: usize>(&mut self) -> [u8; COUNT] {
        let mut tmp: [u8; COUNT] = [0; COUNT];
        tmp.copy_from_slice(&self.buf[self.index..self.index + COUNT]);
        self.index += COUNT;
        tmp
    }

    pub fn read_slice<'a>(&'a mut self, len: usize) -> &'a [u8] {
        let slice = &self.buf[self.index..self.index + len];
        self.index += len;
        slice
    }
}

pub fn encode_u16(buffer: &mut Buffer, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn encode_u24(buffer: &mut Buffer, value: u32) {
    let slice = &value.to_be_bytes();
    buffer.extend_from_slice(&slice[..3]);
}

pub fn encode_u32(buffer: &mut Buffer, value: u32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

pub fn encode_u64(buffer: &mut Buffer, value: u64) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn parse_enumerated<T, E>(bytes: &[u8], len: u32) -> Result<(&[u8], T), T::Error>
where
    T: TryFrom<u32>,
{
    let (bytes, value) = parse_unsigned(bytes, len).unwrap();
    let value = T::try_from(value)?;
    Ok((bytes, value))
}

pub fn decode_unsigned(reader: &mut Reader, len: u32) -> Result<u32, Error> {
    if len > 4 || len == 0 {
        return Err(Error::InvalidValue(
            "unsigned len value is 0 or greater than 4",
        ));
    }

    let val = match len {
        1 => reader.read_byte() as u32,
        2 => u16::from_be_bytes(reader.read_bytes()) as u32,
        3 => {
            // TODO: check this
            let [byte0, byte1, byte2] = reader.read_bytes();
            (byte0 as u32) << 16 | (byte1 as u32) << 8 | (byte2 as u32)
        }
        4 => u32::from_be_bytes(reader.read_bytes()),
        _ => unreachable!(),
    };

    Ok(val)
}

pub fn parse_unsigned(bytes: &[u8], len: u32) -> Result<(&[u8], u32), Error> {
    let len = len as usize;
    if len > 4 || len == 0 {
        return Err(Error::InvalidValue(
            "unsigned len value is 0 or greater than 4",
        ));
    }
    if bytes.len() < len {
        return Err(Error::Length(
            "unsigned len value greater than remaining bytes",
        ));
    }
    let val = match len {
        1 => bytes[0] as u32,
        2 => u16::from_be_bytes(*array_ref!(bytes, 0, 2)) as u32,
        3 => (bytes[0] as u32) << 16 | (bytes[1] as u32) << 8 | bytes[2] as u32,
        4 => u32::from_be_bytes(*array_ref!(bytes, 0, 4)),
        _ => unreachable!(),
    };
    Ok((&bytes[len..], val))
}
