mod enumeration;
mod field;
mod numeric;
mod remainder;

use std::ops::Range;

use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::StringId;
pub use enumeration::EnumSymbol;
pub use field::{FieldSymbol, FieldType};
pub use numeric::*;
pub use remainder::Remainder;
use strum::EnumCount;

use crate::{
    define_ro_struct,
    model::ranges::{Enums, Fields, Messages, Structs},
    table::TypeId,
    utils::Size,
};

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    #[brw(magic = 0u8)]
    Numeric(NumericSymbol),
    #[brw(magic = 1u8)]
    Array(ArraySymbol),
    #[brw(magic = 2u8)]
    Struct(StructSymbol),
    #[brw(magic = 3u8)]
    Enum(EnumSymbol),
}

impl Type {
    #[must_use]
    pub const fn name(&self) -> StringId {
        match self {
            Type::Numeric(sym) => sym.name(),
            Type::Array(_) => unreachable!(),
            Type::Struct(sym) => sym.name(),
            Type::Enum(sym) => sym.name(),
        }
    }
}

define_ro_struct! {
    struct ArraySymbol {
        kind: TypeId,
        len: Size,
    }
}

define_ro_struct! {
    struct StructSymbol {
        name: StringId,
        fields: Fields,
    }
}

define_ro_struct! {
    struct GroupSymbol {
        name: StringId,
        messages: Messages,
        structs: Structs,
        enums: Enums,
    }
}

impl GroupSymbol {
    #[must_use]
    pub const fn type_range(&self) -> Range<usize> {
        let start = self.enums.start() as usize + NumericSymbol::COUNT;
        let len =
            self.structs.start() as usize + self.structs.len() as usize + NumericSymbol::COUNT;
        start..len
    }
}

define_ro_struct! {
    struct MessageSymbol {
        name: StringId,
        fields: Fields,
        remainder: Remainder,
    }
}
