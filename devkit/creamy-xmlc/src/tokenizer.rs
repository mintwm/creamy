use std::fmt::Display;

use roxmltree::{Document, Node, NodeType};

use crate::error::ProtocolError;

#[derive(Debug)]
pub enum Token {
    Protocol {
        name: String,
        version: String,
        access: String,
    },
    Group {
        name: String,
    },
    Message {
        name: String,
    },
    Struct {
        name: String,
    },
    Enum {
        name: String,
    },
    Variant {
        name: String,
    },
    Field {
        name: String,
        kind: String,
    },
    Remainder {
        name: String,
    },
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = match self {
            Token::Protocol { .. } => "protocol",
            Token::Group { .. } => "group",
            Token::Message { .. } => "message",
            Token::Struct { .. } => "struct",
            Token::Enum { .. } => "enum",
            Token::Variant { .. } => "variant",
            Token::Field { .. } => "field",
            Token::Remainder { .. } => "remainder",
        };

        write!(f, "<{token}>")
    }
}

impl Token {
    fn new_protocol(node: Node) -> Result<Token, ProtocolError> {
        Ok(Token::Protocol {
            name: read_attr(node, "name", "protocol")?,
            version: read_attr(node, "version", "protocol")?,
            access: read_attr(node, "access", "protocol")?,
        })
    }

    fn new_group(node: Node) -> Result<Token, ProtocolError> {
        Ok(Token::Group {
            name: read_attr(node, "name", "group")?,
        })
    }

    fn new_message(node: Node) -> Result<Token, ProtocolError> {
        Ok(Token::Message {
            name: read_attr(node, "name", "message")?,
        })
    }

    fn new_struct(node: Node) -> Result<Token, ProtocolError> {
        Ok(Token::Struct {
            name: read_attr(node, "name", "struct")?,
        })
    }

    fn new_enum(node: Node) -> Result<Token, ProtocolError> {
        Ok(Token::Enum {
            name: read_attr(node, "name", "enum")?,
        })
    }

    fn new_field(node: Node) -> Result<Token, ProtocolError> {
        Ok(Token::Field {
            name: read_attr(node, "name", "field")?,
            kind: read_attr(node, "type", "field")?,
        })
    }

    fn new_variant(node: Node) -> Result<Token, ProtocolError> {
        Ok(Token::Variant {
            name: read_attr(node, "name", "variant")?,
        })
    }

    fn new_remainder(node: Node) -> Result<Token, ProtocolError> {
        Ok(Token::Remainder {
            name: read_attr(node, "name", "remainder")?,
        })
    }
}

fn read_attr(node: Node, attr: &str, tag: &str) -> Result<String, ProtocolError> {
    Ok(node
        .attribute(attr)
        .ok_or_else(|| ProtocolError::MissingAttribute {
            tag: tag.to_string(),
            attr: attr.to_string(),
        })?
        .to_string())
}

pub fn tokenize(content: &str) -> Result<Vec<Token>, ProtocolError> {
    let mut tokens = vec![];
    let document = Document::parse(content)?;
    for node in document
        .root()
        .descendants()
        .filter(|n| n.node_type() == NodeType::Element)
    {
        match node.tag_name().name() {
            "protocol" => tokens.push(Token::new_protocol(node)?),
            "group" => tokens.push(Token::new_group(node)?),
            "message" => tokens.push(Token::new_message(node)?),
            "struct" => tokens.push(Token::new_struct(node)?),
            "enum" => tokens.push(Token::new_enum(node)?),
            "field" => tokens.push(Token::new_field(node)?),
            "variant" => tokens.push(Token::new_variant(node)?),
            "remainder" => tokens.push(Token::new_remainder(node)?),
            other => return Err(ProtocolError::UnknownTag(other.to_string())),
        }
    }

    Ok(tokens)
}
