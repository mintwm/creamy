use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::StringId;

use crate::{define_ro_struct, model::symbols::ArraySymbol, table::TypeId};

#[derive(BinRead, BinWrite, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    #[brw(magic(0u8))]
    Type(TypeId),
    #[brw(magic(1u8))]
    Array(ArraySymbol),
}

define_ro_struct! {
    struct FieldSymbol {
        name: StringId,
        kind: FieldType,
    }
}

impl FieldSymbol {
    #[must_use]
    pub const fn type_id(&self) -> TypeId {
        match self.kind {
            FieldType::Type(sym) => sym,
            FieldType::Array(sym) => sym.kind(),
        }
    }
}
