use std::num::NonZeroU8;

use binrw::{BinRead, BinWrite};

use crate::{constraints::MAX_PAYLOAD, error::ProtocolError};

/// This struct is guaranteed that size is non-zero and equal to or less than [`Self::MAX_VALUE`];
#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size(NonZeroU8);
impl Size {
    #[allow(clippy::cast_possible_truncation)]
    pub const MAX_VALUE: u8 = MAX_PAYLOAD as u8;

    pub const B1: Self = Self(NonZeroU8::new(1).unwrap());
    pub const B2: Self = Self(NonZeroU8::new(2).unwrap());
    pub const B4: Self = Self(NonZeroU8::new(4).unwrap());
    pub const B8: Self = Self(NonZeroU8::new(8).unwrap());
    pub const B16: Self = Self(NonZeroU8::new(16).unwrap());

    pub const fn new(size: u8) -> Result<Self, ProtocolError> {
        if size > Self::MAX_VALUE {
            return Err(ProtocolError::InvalidSize(size as usize));
        }
        let Some(value) = NonZeroU8::new(size) else {
            return Err(ProtocolError::InvalidSize(0));
        };
        Ok(Self(value))
    }

    pub const fn value(self) -> u8 {
        self.0.get()
    }
}

impl Default for Size {
    fn default() -> Self {
        Self::B1
    }
}
