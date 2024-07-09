use core::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Phantom(PhantomData<()>);

pub static PHANTOM: Phantom = Phantom(PhantomData {});

#[cfg(feature = "defmt")]
impl defmt::Format for Phantom {
    fn format(&self, _fmt: defmt::Formatter) {
        // do nothing
    }
}

#[cfg(feature = "serde")]
impl<'a, 'de> serde::Deserialize<'de> for &'a Phantom {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(&PHANTOM)
    }
}

impl<'a> Default for &'a Phantom {
    fn default() -> Self {
        &PHANTOM
    }
}
