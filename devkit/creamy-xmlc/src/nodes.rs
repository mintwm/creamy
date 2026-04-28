use creamy_utils::strpool::{NonZeroStringId, StringId, StringPool};

pub use crate::tree::Range;
use crate::{define_ro_struct, error::ProtocolError};

define_ro_struct! {
    [no_brw]
    struct EnumNode {
        name: StringId,
        variants: Range,
    }
}

define_ro_struct! {
    [no_brw]
    struct GroupNode {
        name: StringId,
        messages: Range,
        structs: Range,
        enums: Range,
    }
}

define_ro_struct! {
    [no_brw]
    struct MessageNode {
        name: StringId,
        fields: Range,
        remainder: Option<NonZeroStringId>,
    }
}

define_ro_struct! {
    [no_brw]
    struct StructNode {
        name: StringId,
        fields: Range,
    }
}

define_ro_struct! {
    [no_brw]
    struct ArrayNode {
        kind: StringId,
        size: u8,
    }
}

fn parse_array(input: &str) -> Option<Result<(&str, u8), ProtocolError>> {
    let s = input.trim();

    let l_bracket = s.starts_with('[');
    let r_bracket = s.ends_with(']');

    if (!l_bracket && r_bracket) || (!r_bracket && l_bracket) {
        return Some(Err(ProtocolError::Syntax {
            src: input.to_string(),
            should_be: "[TYPE; SIZE]".to_string(),
        }));
    } else if !l_bracket && !r_bracket {
        return None;
    }

    let content = &s[1..s.len() - 1];
    let mut parts = content.split(';');
    let (type_ident, count) = match (parts.next(), parts.next(), parts.next()) {
        (Some(l), Some(r), None) => (l.trim(), r.trim()),
        _ => {
            return Some(Err(ProtocolError::Syntax {
                src: input.to_string(),
                should_be: "[TYPE; SIZE]".to_string(),
            }));
        }
    };

    Some(match count.parse::<u8>() {
        Ok(count) => Ok((type_ident, count)),
        Err(err) => Err(err.into()),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldTypeNode {
    Type(StringId),
    Array(ArrayNode),
}

impl FieldTypeNode {
    pub fn new(string: &str, pool: &mut StringPool) -> Result<Self, ProtocolError> {
        if let Some(result) = parse_array(string) {
            let (name, size) = result?;
            let name = pool.get_id(name);
            Ok(FieldTypeNode::Array(ArrayNode::new(name, size)))
        } else {
            Ok(FieldTypeNode::Type(pool.get_id(string)))
        }
    }

    pub const fn type_name(self) -> StringId {
        match self {
            FieldTypeNode::Type(id) => id,
            FieldTypeNode::Array(node) => node.kind,
        }
    }
}

define_ro_struct! {
    [no_brw]
    struct FieldNode {
        name: StringId,
        kind: FieldTypeNode,
    }
}
