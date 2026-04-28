use core::{
    fmt::{Debug, Display},
    ops::Deref,
};

use crate::bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Zeroable, Pod, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageType(u16);
impl From<u16> for MessageType {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Deref for MessageType {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for MessageType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl MessageType {
    #[must_use]
    #[inline(always)]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    #[must_use]
    #[inline(always)]
    pub const fn from_le_bytes(bytes: [u8; 2]) -> Self {
        Self(u16::from_le_bytes(bytes))
    }
}

//pub trait Payload: MessageSuper {
//    fn as_bytes(&self) -> &[u8; 28] {
//        bytemuck::cast_ref(self)
//    }
//}

//pub trait MessageSuper: Debug + Copy + Clone {}

pub trait TypedMessage: Debug + Copy + Clone {
    fn dst(&self) -> u8;
    fn with_dst(&mut self, dst: u8) -> &mut Self;
    fn src(&self) -> u8;
    fn group(&self) -> u8;
    fn with_group(&mut self, group: u8) -> &mut Self;
    fn kind(&self) -> u8;
    fn with_kind(&mut self, kind: u8) -> &mut Self;

    //fn as_raw_bytes(&self) -> &[u8; MESSAGE_SIZE];
    //fn payload_as_raw_bytes(&self) -> &[u8; PAYLOAD_SIZE];

    #[must_use]
    #[inline(always)]
    fn cast<M: TypedMessage>(self) -> M {
        const {
            assert!(size_of::<Self>() == size_of::<M>());
            assert!(align_of::<Self>() == align_of::<M>());
        }
        unsafe { *core::ptr::from_ref(&self).cast::<M>() }
    }
}
