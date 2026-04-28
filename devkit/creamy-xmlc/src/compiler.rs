use std::num::NonZeroU8;

use creamy_utils::strpool::StringPool;

use crate::{
    Access, ProtocolDefinition, Version,
    constraints::{MAX_FIELDS, MAX_MESSAGES},
    error::{ProtocolError, ProtocolErrorExt},
    model::{
        ranges::{Enums, Fields, Messages, Structs, Variants},
        symbols::{
            ArraySymbol, EnumSymbol, FieldSymbol, FieldType, GroupSymbol, MessageSymbol, Remainder,
            StructSymbol, Type,
        },
    },
    nodes::{EnumNode, FieldNode, FieldTypeNode, MessageNode, StructNode},
    table::TypeTable,
    tokenizer::tokenize,
    tree::ProtocolTree,
    utils::{BoundedVec, Size},
};

pub struct ProtocolCompiler<'p> {
    pool: &'p mut StringPool,
}

impl<'p> ProtocolCompiler<'p> {
    pub const fn new(pool: &'p mut StringPool) -> Self {
        Self { pool }
    }

    pub fn compile(&mut self, content: &str) -> Result<ProtocolDefinition, Vec<ProtocolError>> {
        let tokens = tokenize(content.trim()).map_err(|e| vec![e])?;
        let tree = ProtocolTree::new(tokens, self.pool).map_err(|e| vec![e])?;
        Self::run(tree)
    }

    fn run(mut tree: ProtocolTree) -> Result<ProtocolDefinition, Vec<ProtocolError>> {
        let mut errors = Vec::new();

        //TODO 'as' overflow checks
        let mut tt = TypeTable::new(tree.groups.len() as u8, tree.type_count());

        let version = tree.version.parse::<Version>().or_save_to(&mut errors);
        let access = Access::new(&tree.access).or_save_to(&mut errors);

        let variants = std::mem::take(&mut tree.variants);
        let mut groups = BoundedVec::with_capacity(tree.groups.len());
        let mut fields = BoundedVec::with_capacity(tree.fields.len());
        let mut messages = BoundedVec::with_capacity(tree.messages.len());

        for (idx, group) in tree.groups.drain(..).enumerate() {
            let g_id = NonZeroU8::new(idx as u8 + 1).expect("Group limit exceeded");
            let mut resolver = Resolver::new(&mut errors, &mut tt, g_id);

            resolver.resolve_enums(&tree.enums[group.enums().as_range()]);
            resolver.resolve_structs(
                &tree.structs[group.structs().as_range()],
                &tree.fields,
                &mut fields,
            );

            resolver.resolve_messages(
                &tree.messages[group.messages().as_range()],
                &mut messages,
                &tree.fields,
                &mut fields,
            );

            let messages = {
                let range = group.messages();
                Messages::new(range.start() as u8, range.len() as u8)
            };

            let structs = {
                let range = group.structs();
                Structs::new(range.start() as u32, range.len() as u8)
            };

            let enums = {
                let range = group.enums();
                Enums::new(range.start() as u32, range.len() as u8)
            };

            assert!(
                groups.push(GroupSymbol::new(group.name(), messages, structs, enums)),
                "unreachable!"
            );
        }

        if errors.is_empty() {
            Ok(ProtocolDefinition::new(
                tree.name,
                version,
                access,
                variants,
                fields,
                groups,
                messages,
                tt.finish(),
            ))
        } else {
            Err(errors)
        }
    }
}

//TODO: validate size
//TODO: remove unused
//TODO: errors
//TODO: warnings
//TODO: executable
//TODO: suggest best layout
//TODO: name duplicate

struct Resolver<'a> {
    errors: &'a mut Vec<ProtocolError>,
    tt: &'a mut TypeTable,
    group: NonZeroU8,
}

impl<'a> Resolver<'a> {
    const fn new(
        errors: &'a mut Vec<ProtocolError>,
        tt: &'a mut TypeTable,
        group: NonZeroU8,
    ) -> Self {
        Self { errors, tt, group }
    }

    fn resolve_field(&mut self, from: &[FieldNode], to: &mut BoundedVec<FieldSymbol, MAX_FIELDS>) {
        for field in from {
            let kind = match field.kind() {
                FieldTypeNode::Type(name) => {
                    FieldType::Type(self.tt.get_type_by_name(name).unwrap())
                }
                FieldTypeNode::Array(array) => {
                    let kind = self.tt.get_type_by_name(array.kind()).unwrap();
                    let size = Size::new(array.size());
                    FieldType::Array(ArraySymbol::new(kind, size.or_save_to(self.errors)))
                }
            };
            assert!(to.push(FieldSymbol::new(field.name(), kind)));
        }
    }

    fn resolve_enums(&mut self, from: &[EnumNode]) {
        for e in from {
            let range = e.variants();
            let variants = Variants::new(range.start() as u16, range.len() as u16);
            let sym = EnumSymbol::new(e.name(), variants);
            let meta = sym.meta().or_save_to(self.errors);
            self.tt.register_type(self.group, meta, Type::Enum(sym));
        }
    }

    fn resolve_structs(
        &mut self,
        structs: &[StructNode],
        f_from: &[FieldNode],
        f_to: &mut BoundedVec<FieldSymbol, MAX_FIELDS>,
    ) {
        let mut to_resolve = structs.len();
        // Устанавливаем любое значение, лишь бы было не 0
        let mut last_resolved = 1;
        while to_resolve != 0 {
            for s in structs {
                let range = s.fields();
                let from = &f_from[range.as_range()];
                if from
                    .iter()
                    .any(|f| !self.tt.contains_name(f.kind().type_name()))
                {
                    assert!(last_resolved != 0, "cannot resolve struct");

                    continue;
                }

                if self.tt.contains_name(s.name()) {
                    continue;
                }

                self.resolve_field(from, f_to);
                let len = f_to.len() as usize;
                let meta = ProtocolDefinition::get_struct_meta(&f_to[len - range.len()..len])
                    .or_save_to(self.errors);
                let s = StructSymbol::new(
                    s.name(),
                    Fields::new(range.start() as u32, range.len() as u8),
                );

                self.tt.register_type(self.group, meta, Type::Struct(s));

                to_resolve -= 1;
                last_resolved = 0;
            }
        }
    }

    fn resolve_messages(
        &mut self,
        m_from: &[MessageNode],
        m_to: &mut BoundedVec<MessageSymbol, MAX_MESSAGES>,
        f_from: &[FieldNode],
        f_to: &mut BoundedVec<FieldSymbol, MAX_FIELDS>,
    ) {
        for m in m_from {
            let range = m.fields();
            let from = &f_from[range.as_range()];
            self.resolve_field(from, f_to);

            let fields = Fields::new(range.start() as u32, range.len() as u8);
            let len = f_to.len() as usize;

            // Check size and align
            ProtocolDefinition::get_struct_meta(&f_to[len - range.len()..len])
                .or_save_to(self.errors);

            let sym = MessageSymbol::new(m.name(), fields, Remainder::new(m.remainder()));

            assert!(m_to.push(sym), "unreachable!");
        }
    }
}
