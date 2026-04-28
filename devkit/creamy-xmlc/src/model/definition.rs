use std::fmt::Display;

use as_guard::AsGuard;
use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::StringId;

use crate::{
    Version,
    constraints::{HEADER_BYTES, MAX_FIELDS, MAX_GROUPS, MAX_MESSAGES, MAX_PAYLOAD, MAX_VARIANTS},
    error::ProtocolError,
    model::{
        ranges::{Fields, Messages, Variants},
        symbols::{FieldSymbol, GroupSymbol, MessageSymbol, Type},
    },
    table::{FinishedTypeTable, TypeMeta},
    utils::BoundedVec,
};

#[derive(BinRead, BinWrite, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Access {
    #[default]
    #[brw(magic(0u8))]
    ExclusiveWrite,
    #[brw(magic(1u8))]
    MultipleWrite,
}

impl Access {
    pub fn new(string: &str) -> Result<Access, ProtocolError> {
        match string {
            "ExclusiveWrite" => Ok(Access::ExclusiveWrite),
            "MultipleWrite" => Ok(Access::MultipleWrite),
            value => Err(ProtocolError::InvalidAccess(value.to_string())),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Access::ExclusiveWrite => "ExclusiveWrite",
            Access::MultipleWrite => "MultipleWrite",
        };
        write!(f, "{str}")
    }
}

#[derive(BinRead, BinWrite, Debug, PartialEq, Eq)]
pub struct ProtocolDefinition {
    name: StringId,
    version: Version,
    access: Access,
    variants: BoundedVec<StringId, MAX_VARIANTS>,
    fields: BoundedVec<FieldSymbol, MAX_FIELDS>,
    groups: BoundedVec<GroupSymbol, MAX_GROUPS>,
    messages: BoundedVec<MessageSymbol, MAX_MESSAGES>,
    table: FinishedTypeTable,
}

impl ProtocolDefinition {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        name: StringId,
        version: Version,
        access: Access,
        variants: BoundedVec<StringId, MAX_VARIANTS>,
        fields: BoundedVec<FieldSymbol, MAX_FIELDS>,
        groups: BoundedVec<GroupSymbol, MAX_GROUPS>,
        messages: BoundedVec<MessageSymbol, MAX_MESSAGES>,
        table: FinishedTypeTable,
    ) -> Self {
        Self {
            name,
            version,
            access,
            variants,
            fields,
            groups,
            messages,
            table,
        }
    }

    #[must_use]
    pub const fn name(&self) -> StringId {
        self.name
    }

    #[must_use]
    pub const fn name_ref(&self) -> &StringId {
        &self.name
    }

    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    #[must_use]
    pub const fn access(&self) -> Access {
        self.access
    }

    #[must_use]
    pub fn groups(&self) -> &[GroupSymbol] {
        self.groups.as_slice()
    }

    #[must_use]
    pub fn messages(&self) -> &[MessageSymbol] {
        self.messages.as_slice()
    }

    #[must_use]
    pub const fn table(&self) -> &FinishedTypeTable {
        &self.table
    }

    #[must_use]
    pub fn message_slice(&self, messages: Messages) -> &[MessageSymbol] {
        let start = messages.start() as usize;
        let len = messages.len() as usize;
        &self.messages[start..start + len]
    }

    #[must_use]
    pub fn field_slice(&self, fields: Fields) -> &[FieldSymbol] {
        let start = fields.start() as usize;
        let len = fields.len() as usize;
        &self.fields[start..start + len]
    }

    #[must_use]
    pub fn get_struct_paddings(fields: &[FieldSymbol]) -> u8 {
        let mut total_size = HEADER_BYTES;
        let mut paddings = 0;
        for field in fields {
            let tid = field.type_id();
            let size = tid.size().value();
            let align = tid.align().value();

            let padding = (align - (total_size % align)) % align;
            paddings += padding;

            total_size += padding + size;
        }

        paddings
    }

    pub fn get_struct_meta(fields: &[FieldSymbol]) -> Result<TypeMeta, ProtocolError> {
        let mut total_size = HEADER_BYTES as usize;
        let mut max_align = 1;

        for field in fields {
            let tid = field.type_id();
            let size = tid.size().value() as usize;
            let align = tid.align().value() as usize;

            if align > max_align {
                max_align = align;
            }

            let padding = (align - (total_size % align)) % align;

            total_size += padding + size;
        }

        let tail_padding = (max_align - (total_size % max_align)) % max_align;
        total_size += tail_padding;
        total_size -= HEADER_BYTES as usize;

        if total_size > MAX_PAYLOAD {
            return Err(ProtocolError::InvalidSize(total_size));
        }

        assert!(u8::try_from(max_align).is_ok());

        TypeMeta::new(total_size.safe_as::<u8>(), max_align.safe_as::<u8>())
    }

    pub fn message_iter(&self, mut f: impl FnMut(GroupSymbol, MessageSymbol)) {
        self.groups
            .iter()
            .flat_map(|g| self.message_slice(g.messages()).iter().map(move |m| (g, m)))
            .for_each(|(g, m)| f(*g, *m));
    }

    pub fn group_iter<E>(
        &self,
        mut f: impl FnMut(GroupSymbol, &[MessageSymbol], &[Type]) -> Result<(), E>,
    ) -> Result<(), E> {
        self.groups
            .iter()
            .map(|g| {
                (
                    g,
                    self.message_slice(g.messages()),
                    &self.table.types()[g.type_range()],
                )
            })
            .try_for_each(|(g, m, t)| f(*g, m, t))
    }

    #[must_use]
    pub fn variant_slice(&self, variants: Variants) -> &[StringId] {
        let start = variants.start() as usize;
        let len = variants.len() as usize;
        &self.variants[start..start + len]
    }
}

pub struct LayoutStep {
    pub size: u8,
    pub align: u8,
    pub padding: u8,
    pub offset: u8,
}

pub fn compute_layout<E>(
    fields: &[FieldSymbol],
    mut f: impl FnMut(&FieldSymbol, LayoutStep) -> Result<(), E>,
) -> Result<u8, E> {
    let mut total_size = HEADER_BYTES;

    for field in fields {
        let tid = field.type_id();
        let size = tid.size().value();
        let align = tid.align().value();

        let padding = (align - (total_size % align)) % align;

        f(
            field,
            LayoutStep {
                size,
                align,
                padding,
                offset: total_size + padding,
            },
        )?;

        total_size += padding + size;
    }

    Ok(total_size)
}
