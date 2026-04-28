#![allow(clippy::as_conversions)]
#![allow(clippy::cast_possible_truncation)]

use std::ops::{Deref, DerefMut};

use binrw::{BinRead, BinWrite};

#[derive(Debug, PartialEq, Eq)]
pub struct BoundedVec<T, const MAX: usize> {
    inner: Vec<T>,
}

impl<T, const MAX: usize> Default for BoundedVec<T, MAX> {
    fn default() -> Self {
        Self { inner: Vec::new() }
    }
}

impl<T, const MAX: usize> BoundedVec<T, MAX> {
    pub const fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn with_capacity(capacity: u32) -> Self {
        //TODO check
        Self {
            inner: Vec::with_capacity(capacity as usize),
        }
    }

    /// # Returns
    /// Возвращает ``true`` если элемент успешно добавлен, иначе ``false``
    #[must_use]
    pub fn push(&mut self, item: T) -> bool {
        if self.inner.len() >= MAX {
            return false;
        }

        self.inner.push(item);
        true
    }

    pub const fn len(&self) -> u32 {
        self.inner.len() as u32
    }

    pub const fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }
}

impl<T, const MAX: usize> DerefMut for BoundedVec<T, MAX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T, const MAX: usize> Deref for BoundedVec<T, MAX> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: BinRead, const MAX: usize> BinRead for BoundedVec<T, MAX>
where
    T: for<'a> BinRead<Args<'a> = ()>,
{
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let len = u32::read_options(reader, endian, args)? as usize;
        let mut inner = Vec::with_capacity(len);
        for _ in 0..len {
            inner.push(T::read_options(reader, endian, args)?);
        }

        Ok(Self { inner })
    }
}

impl<T: BinWrite, const MAX: usize> BinWrite for BoundedVec<T, MAX>
where
    T: for<'a> BinWrite<Args<'a> = ()>,
{
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        (self.inner.len() as u32).write_options(writer, endian, args)?;
        let slice = &self.inner[..self.inner.len()];
        for item in slice {
            item.write_options(writer, endian, args)?;
        }
        Ok(())
    }
}
