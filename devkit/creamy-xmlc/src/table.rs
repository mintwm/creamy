use std::{collections::HashMap, mem::MaybeUninit, num::NonZeroU8};

use binrw::{BinRead, BinWrite, binrw};
use creamy_utils::{collections::Array, strpool::StringId};
use strum::EnumCount;

use crate::{
    error::ProtocolError,
    model::symbols::{
        BUILTIN_GROUP, NumericSymbol, Type, U8_META, U16_META, U32_META, U64_META, U128_META,
    },
    utils::{Align, Size},
};

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeMeta {
    /// See [Align] for valid values
    /// * 00000 - size (max 28) `[11100_XXX]`
    /// * 000   - align (max 3) `[XXXXX_011]`
    value: u8,
}

impl Default for TypeMeta {
    fn default() -> Self {
        Self::new_with_assert(Size::B1, Align::B1)
    }
}

impl TypeMeta {
    /// * Size: 0 < ``size`` <= 28
    /// * Align: ``[1, 2, 4, 8]``
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const fn new(size: u8, align: u8) -> Result<Self, ProtocolError> {
        if size == 0 || size > Size::MAX_VALUE {
            return Err(ProtocolError::InvalidSize(size as usize));
        }

        if align > 8 {
            return Err(ProtocolError::ForbiddenAlign(align));
        }

        if !align.is_power_of_two() {
            return Err(ProtocolError::AlignIsNotPowerOfTwo(align));
        }

        let size = size << 3;
        let value = size | Align::to_raw(align);
        Ok(Self { value })
    }

    /// * Size: 0 < ``size`` <= 28
    /// * Align: ``[0, 1, 2, 3]``
    const fn new_raw(size: u8, align: u8) -> Result<Self, ProtocolError> {
        if size == 0 || size > Size::MAX_VALUE {
            return Err(ProtocolError::InvalidSize(size as usize));
        }
        if align > 3 {
            return Err(ProtocolError::ForbiddenRawAlign(align));
        }

        let size = size << 3;
        let value = size | align;
        Ok(Self { value })
    }

    pub const fn new_typed(size: Size, align: Align) -> Result<Self, ProtocolError> {
        let size = size.value();
        let align = align.raw_value();
        Self::new_raw(size, align)
    }

    const fn new_raw_with_assert(size: u8, align: u8) -> Self {
        assert!(size <= Size::MAX_VALUE);
        assert!(align <= 3);
        let size = size << 3;
        let value = size | align;
        Self { value }
    }

    pub(crate) const fn new_with_assert(size: Size, align: Align) -> Self {
        let size = size.value();
        let align = align.raw_value();
        Self::new_raw_with_assert(size, align)
    }

    pub fn size(self) -> Size {
        Size::new(self.value >> 3).expect("unreachable!")
    }

    pub const fn align(self) -> Align {
        let value = self.value & 0b00000111;
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeId {
    index: u16,
    group: NonZeroU8,
    meta: TypeMeta,
}

impl TypeId {
    pub(crate) const fn new(group: NonZeroU8, id: StringId, meta: TypeMeta) -> Self {
        TypeId {
            group,
            index: id.value(),
            meta,
        }
    }

    pub fn size(self) -> Size {
        self.meta.size()
    }

    pub const fn align(self) -> Align {
        self.meta.align()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct GroupsMeta {
    count: u8,
    inner: Array<u16>,
}

impl GroupsMeta {
    fn next_free_index(&self, ty: TypeId) -> usize {
        let group = ty.group.get() as usize;
        let index = self.inner[group] as usize;

        let start_of_group: usize = self.inner[..group]
            .iter()
            .copied()
            .map(|v| v as usize)
            .sum();
        start_of_group + index
    }

    fn position_of(&self, ty: TypeId) -> usize {
        let group = ty.group.get() as usize;
        let start_of_group: usize = self.inner[..group]
            .iter()
            .copied()
            .map(|v| v as usize)
            .sum();
        start_of_group + ty.index as usize
    }

    fn total_used(&self) -> usize {
        self.inner[..=self.count as usize]
            .iter()
            .copied()
            .map(|v| v as usize)
            .sum()
    }
}

impl BinRead for GroupsMeta {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let count_u8 = u8::read_options(reader, endian, args)?;
        let count = count_u8 as usize + 1;
        let mut inner = Array::new_with_default(count, 0);
        for index in 0..count {
            inner[index] = u16::read_options(reader, endian, args)?;
        }
        Ok(Self {
            inner,
            count: count_u8,
        })
    }
}

impl BinWrite for GroupsMeta {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        (self.count).write_options(writer, endian, args)?;
        let payload = &self.inner[..=(self.count as usize)];
        for &count in payload {
            count.write_options(writer, endian, args)?;
        }
        Ok(())
    }
}

//TODO: replace u8 and NonZeroU8 with InternalU8 and ExternalU8

pub struct TypeTable {
    meta: GroupsMeta,
    types: Array<MaybeUninit<Type>>,

    //Fix 'same Id's but different types'
    inner: HashMap<StringId, TypeId>,
}

impl TypeTable {
    pub fn new(groups: u8, types: u16) -> Self {
        //TODO: fix use internal and external u8
        let groups = groups.saturating_add(1);
        let types = types as usize + NumericSymbol::COUNT;

        let mut instance = Self {
            meta: GroupsMeta {
                count: groups,
                inner: Array::new_with_default(256, 0),
            },
            types: Array::zeroed(types),
            inner: HashMap::new(),
        };

        instance.register_type(BUILTIN_GROUP, U8_META, Type::Numeric(NumericSymbol::U8));
        instance.register_type(BUILTIN_GROUP, U16_META, Type::Numeric(NumericSymbol::U16));
        instance.register_type(BUILTIN_GROUP, U32_META, Type::Numeric(NumericSymbol::U32));
        instance.register_type(BUILTIN_GROUP, U64_META, Type::Numeric(NumericSymbol::U64));
        instance.register_type(BUILTIN_GROUP, U128_META, Type::Numeric(NumericSymbol::U128));

        instance.register_type(BUILTIN_GROUP, U8_META, Type::Numeric(NumericSymbol::I8));
        instance.register_type(BUILTIN_GROUP, U16_META, Type::Numeric(NumericSymbol::I16));
        instance.register_type(BUILTIN_GROUP, U32_META, Type::Numeric(NumericSymbol::I32));
        instance.register_type(BUILTIN_GROUP, U64_META, Type::Numeric(NumericSymbol::I64));
        instance.register_type(BUILTIN_GROUP, U128_META, Type::Numeric(NumericSymbol::I128));

        instance.register_type(BUILTIN_GROUP, U32_META, Type::Numeric(NumericSymbol::F32));
        instance.register_type(BUILTIN_GROUP, U64_META, Type::Numeric(NumericSymbol::F64));

        instance
    }

    pub fn get_type_by_name(&self, name: StringId) -> Option<TypeId> {
        self.inner.get(&name).copied()
    }

    pub fn register_type(&mut self, group: NonZeroU8, meta: TypeMeta, sym: Type) {
        let group_usize = group.get() as usize;

        let index = self.meta.inner[group_usize];

        let id = TypeId { index, group, meta };
        let flat_index = self.meta.next_free_index(id);

        let name = sym.name();
        self.types[flat_index] = MaybeUninit::new(sym);
        assert!(self.inner.insert(name, id).is_none());

        // Increase offset
        self.meta.inner[group_usize] += 1;
    }

    /*
       pub fn contains(&self, ty: TypeId) -> bool {
           let pos = self.meta.position_of(ty);
           let value = &self.types[pos];

           unsafe {
               let ptr = &value as *const _ as *const u8;
               let size = std::mem::size_of::<Type>();
               let bytes = std::slice::from_raw_parts(ptr, size);

               bytes.iter().any(|&b| b != 0)
           }
       }
    */
    pub fn contains_name(&self, name: StringId) -> bool {
        self.inner.contains_key(&name)
    }

    pub fn finish(mut self) -> FinishedTypeTable {
        assert_eq!(self.meta.total_used(), self.types.len());
        //self.types.trim(self.meta.total_used());
        let types = unsafe { self.types.assume_init() };
        self.meta.inner.trim(self.meta.count as usize + 1);
        FinishedTypeTable {
            meta: self.meta,
            types,
        }
    }
}

#[binrw]
#[derive(Debug, PartialEq, Eq)]
pub struct FinishedTypeTable {
    meta: GroupsMeta,
    types: Array<Type>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl FinishedTypeTable {
    #[must_use]
    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    #[must_use]
    pub const fn group_count(&self) -> usize {
        self.meta.count as usize
    }

    #[must_use]
    pub fn types(&self) -> &[Type] {
        &self.types
    }

    #[must_use]
    pub fn get_type(&self, id: TypeId) -> &Type {
        &self.types[self.meta.position_of(id)]
    }

    #[must_use]
    pub fn name_of_type(&self, id: TypeId) -> StringId {
        self.get_type(id).name()
    }
}
