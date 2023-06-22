use super::error::Error;

pub const BACNET_MAX_OBJECT: u32 = 0x3FF;
pub const BACNET_INSTANCE_BITS: u32 = 22;
pub const BACNET_MAX_INSTANCE: u32 = 0x3FFFFF;
pub const MAX_BITSTRING_BYTES: u32 = 15;
pub const BACNET_ARRAY_ALL: u32 = 0xFFFFFFFF;
pub const BACNET_NO_PRIORITY: u32 = 0;
pub const BACNET_MIN_PRIORITY: u32 = 1;
pub const BACNET_MAX_PRIORITY: u32 = 16;

#[derive(Debug)]
#[repr(u32)]
pub enum Segmentation {
    Both = 0,
    Transmit = 1,
    Receive = 2,
    None = 3,
    Max = 4,
}

impl TryFrom<u32> for Segmentation {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Both),
            1 => Ok(Self::Transmit),
            2 => Ok(Self::Receive),
            3 => Ok(Self::None),
            4 => Ok(Self::Max),
            _ => Err(Error::InvalidValue("invalid segmentation value")),
        }
    }
}
