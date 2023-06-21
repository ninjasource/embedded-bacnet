use super::error::Error;

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
