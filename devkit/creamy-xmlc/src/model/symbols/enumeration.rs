use creamy_utils::strpool::StringId;

use crate::{
    define_ro_struct,
    error::ProtocolError,
    model::{ranges::Variants, symbols::NumericSymbol},
    table::TypeMeta,
};

define_ro_struct! {
    struct EnumSymbol {
        name: StringId,
        variants: Variants,
    }
}

impl EnumSymbol {
    const fn get_numeric_type(self) -> NumericSymbol {
        if self.variants.len() <= u8::MAX as u16 {
            NumericSymbol::U8
        } else {
            NumericSymbol::U16
        }
    }

    pub const fn meta(&self) -> Result<TypeMeta, ProtocolError> {
        let ty = self.get_numeric_type();
        let size = ty.size();
        let align = ty.align();
        TypeMeta::new(size, align)
    }
}
