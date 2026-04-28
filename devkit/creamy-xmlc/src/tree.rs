use creamy_utils::strpool::{StringId, StringPool};

use crate::{
    StringPoolIntern,
    constraints::{MAX_ENUMS, MAX_FIELDS, MAX_GROUPS, MAX_MESSAGES, MAX_STRUCTS, MAX_VARIANTS},
    error::ProtocolError,
    nodes::{EnumNode, FieldNode, FieldTypeNode, GroupNode, MessageNode, StructNode},
    tokenizer::Token,
    utils::BoundedVec,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    start: usize,
    len: usize,
}

impl Range {
    pub const fn start(&self) -> usize {
        self.start
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn as_range(&self) -> core::ops::Range<usize> {
        self.start..self.start + self.len
    }
}

#[derive(Default)]
struct RangeBuilder {
    start: usize,
    len: usize,
}

impl RangeBuilder {
    const fn next(&mut self) {
        self.len += 1;
    }

    const fn build(&mut self) -> Range {
        let range = Range {
            start: self.start,
            len: self.len,
        };
        self.start += self.len;
        self.len = 0;
        range
    }
}

#[derive(Debug)]
pub struct ProtocolTree {
    pub name: StringId,
    pub version: String,
    pub access: String,

    pub groups: BoundedVec<GroupNode, MAX_GROUPS>,
    pub messages: BoundedVec<MessageNode, MAX_MESSAGES>,
    pub fields: BoundedVec<FieldNode, MAX_FIELDS>,
    pub structs: BoundedVec<StructNode, MAX_STRUCTS>,
    pub enums: BoundedVec<EnumNode, MAX_ENUMS>,
    pub variants: BoundedVec<StringId, MAX_VARIANTS>,
}

impl ProtocolTree {
    #[allow(clippy::too_many_lines)]
    pub fn new(mut tokens: Vec<Token>, pool: &mut StringPool) -> Result<Self, ProtocolError> {
        let Token::Protocol {
            name,
            version,
            access,
        } = tokens.remove(0)
        else {
            return Err(ProtocolError::MissingProtocolToken);
        };

        let mut groups = BoundedVec::new();
        let mut messages = BoundedVec::new();
        let mut fields = BoundedVec::new();
        let mut structs = BoundedVec::new();
        let mut enums = BoundedVec::new();
        let mut variants = BoundedVec::new();

        let mut variant_builder = RangeBuilder::default();
        let mut field_builder = RangeBuilder::default();
        let mut message_range_builder = RangeBuilder::default();
        let mut struct_range_builder = RangeBuilder::default();
        let mut enum_range_builder = RangeBuilder::default();
        let mut group_name: Option<StringId> = None;

        let mut iter = tokens.drain(..).peekable();

        while let Some(token) = iter.next() {
            match token {
                Token::Group { name } => {
                    if let Some(name) = group_name.take() {
                        let messages = message_range_builder.build();
                        let structs = struct_range_builder.build();
                        let enums = enum_range_builder.build();
                        if !groups.push(GroupNode::new(name, messages, structs, enums)) {
                            return Err(ProtocolError::TooManyGroups);
                        }
                    }

                    group_name = Some(name.intern(pool));
                }
                Token::Message { name } => {
                    let mut remainder = None;

                    while let Some(Token::Field { name, kind }) =
                        iter.next_if(|t| matches!(t, Token::Field { .. }))
                    {
                        let node =
                            FieldNode::new(name.intern(pool), FieldTypeNode::new(&kind, pool)?);
                        if !fields.push(node) {
                            return Err(ProtocolError::TooManyFields);
                        }
                        field_builder.next();
                    }

                    if let Some(Token::Remainder { name }) =
                        iter.next_if(|t| matches!(t, Token::Remainder { .. }))
                    {
                        remainder = Some(name.intern_non_zero(pool));
                    }

                    message_range_builder.next();
                    if !messages.push(MessageNode::new(
                        name.intern(pool),
                        field_builder.build(),
                        remainder,
                    )) {
                        return Err(ProtocolError::TooManyMessages);
                    }
                }
                Token::Struct { name } => {
                    while let Some(Token::Field { name, kind }) =
                        iter.next_if(|t| matches!(t, Token::Field { .. }))
                    {
                        let field =
                            FieldNode::new(name.intern(pool), FieldTypeNode::new(&kind, pool)?);

                        if !fields.push(field) {
                            return Err(ProtocolError::TooManyFields);
                        }
                        field_builder.next();
                    }

                    struct_range_builder.next();
                    if !structs.push(StructNode::new(name.intern(pool), field_builder.build())) {
                        return Err(ProtocolError::TooManyStructs);
                    }
                }
                Token::Enum { name } => {
                    while let Some(Token::Variant { name }) =
                        iter.next_if(|t| matches!(t, Token::Variant { .. }))
                    {
                        if !variants.push(name.intern(pool)) {
                            return Err(ProtocolError::TooManyVariants);
                        }
                        variant_builder.next();
                    }

                    enum_range_builder.next();
                    if !enums.push(EnumNode::new(name.intern(pool), variant_builder.build())) {
                        return Err(ProtocolError::TooManyEnums);
                    }
                }
                other => return Err(ProtocolError::UnexpectedToken(other.to_string())),
            }
        }

        if let Some(name) = group_name.take() {
            let messages = message_range_builder.build();
            let structs = struct_range_builder.build();
            let enums = enum_range_builder.build();
            if !groups.push(GroupNode::new(name, messages, structs, enums)) {
                return Err(ProtocolError::TooManyGroups);
            }
        }

        Ok(ProtocolTree {
            name: pool.get_id(&name),
            version,
            access,

            groups,
            messages,
            fields,
            structs,
            enums,
            variants,
        })
    }

    /// Значение не может превышать [``crate::constraints::MAX_TYPE_COUNT``]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn type_count(&self) -> u16 {
        (self.enums.len() + self.structs.len()) as u16
    }
}
