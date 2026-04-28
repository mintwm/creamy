#![allow(clippy::missing_errors_doc)]

use std::{borrow::Cow, fmt::Display};

use creamy_utils::strpool::StringPool;
use creamy_xmlc::{
    ProtocolDefinition, StringPoolResolver,
    model::{
        definition::compute_layout,
        symbols::{FieldSymbol, Type},
    },
};

#[derive(Default, Clone, Copy)]
pub struct Args {
    pub eq: bool,
    pub ord: bool,
    pub hash: bool,
}

const TYPED_MESSAGE: &str = "cbus_core::message::TypedMessage";
//const MESSAGE_SIZE: &str = "cbus_core::defines::MESSAGE_SIZE";

fn extend_derive(list: &mut DeriveList, args: Args) {
    list.inner.extend_from_slice(&["Debug", "Copy", "Clone"]);

    if args.eq {
        list.inner.extend_from_slice(&["PartialEq", "Eq"]);
    }

    if args.ord {
        list.inner.extend_from_slice(&["PartialOrd", "Ord"]);
    }

    if args.hash {
        list.inner.push("Hash");
    }
}

#[allow(clippy::too_many_lines)]
pub fn generate<W: std::io::Write>(
    writer: &mut W,
    args: Args,
    pool: &StringPool,
    definition: &ProtocolDefinition,
) -> Result<(), std::io::Error> {
    definition.group_iter::<std::io::Error>(|group, messages, types| {
        let mut module = Module::new(group.name().resolve(pool));
        module.access = Access::Pub;

        for ty in types {
            match ty {
                Type::Numeric(_) | Type::Array(_) => {}
                Type::Struct(sym) => {
                    let mut struct_ = Struct::new(sym.name().resolve(pool));
                    struct_.access = Access::Pub;
                    extend_derive(&mut struct_.derives, args);
                    struct_.fields = definition
                        .field_slice(sym.fields())
                        .iter()
                        .map(|f| {
                            let ty = definition.table().get_type(f.type_id());
                            Field {
                                access: Access::Pub,
                                name: Cow::Borrowed(f.name().resolve(pool)),
                                kind: Cow::Borrowed(ty.name().resolve(pool)),
                                comment: None,
                            }
                        })
                        .collect();

                    module.structs.push(struct_);
                }
                Type::Enum(sym) => {
                    let mut enum_ = Enum::new(sym.name().resolve(pool));
                    enum_.access = Access::Pub;
                    extend_derive(&mut enum_.derives, args);
                    enum_.variants = definition
                        .variant_slice(sym.variants())
                        .iter()
                        .map(|v| v.resolve(pool))
                        .collect();

                    module.enums.push(enum_);
                }
            }
        }

        for message in messages {
            let message_name = message.name().resolve(pool);
            let mut struct_ = Struct::new(message_name);
            struct_.access = Access::Pub;
            struct_.add_repr = true;
            extend_derive(&mut struct_.derives, args);

            struct_.fields.push(Field {
                access: Access::Pub,
                name: Cow::Borrowed("dst"),
                kind: Cow::Borrowed("u8"),
                comment: Some(Cow::Borrowed("-------- HEADER --------")),
            });

            struct_.fields.push(Field {
                access: Access::Pub,
                name: Cow::Borrowed("group"),
                kind: Cow::Borrowed("u8"),
                comment: None,
            });

            struct_.fields.push(Field {
                access: Access::Pub,
                name: Cow::Borrowed("src"),
                kind: Cow::Borrowed("u8"),
                comment: None,
            });

            struct_.fields.push(Field {
                access: Access::Pub,
                name: Cow::Borrowed("kind"),
                kind: Cow::Borrowed("u8"),
                comment: None,
            });

            let slice = definition.field_slice(message.fields());
            let mut paddings = 0;
            let total_size = compute_layout(slice, |f, l| {
                if l.padding != 0 {
                    struct_.fields.push(Field {
                        access: Access::Pub,
                        name: Cow::Owned(format!("_padding{paddings}")),
                        kind: Cow::Owned(format!("[u8; {}]", l.padding)),
                        comment: None,
                    });
                    paddings += 1;
                }

                let ty = definition.table().get_type(f.type_id());
                struct_.fields.push(Field {
                    access: Access::Pub,
                    name: Cow::Borrowed(f.name().resolve(pool)),
                    kind: Cow::Borrowed(ty.name().resolve(pool)),
                    comment: None,
                });

                Result::<(), std::io::Error>::Ok(())
            })?;

            let diff = 32 - total_size;
            if diff != 0 {
                struct_.fields.push(Field {
                    access: Access::Pub,
                    name: Cow::Owned(format!("_padding{paddings}")),
                    kind: Cow::Owned(format!("[u8; {diff}]")),
                    comment: None,
                });
            }

            struct_.fields[4].comment = Some(Cow::Borrowed("-------- PAYLOAD --------"));

            //writeln!(writer, "    /* ------------------------ */")?;
            //writeln!(writer, "    /* ------------------------- */")?;

            //generate_assert_impl(&mut writer, message.name().resolve(pool))?;
            //generate_builder_pattern(
            //    &mut writer,
            //    pool,
            //    definition,
            //    message.name().resolve(pool),
            //    slice,
            //)?;

            struct_
                .trait_impls
                .push(generate_message_trait_impl(message_name));

            struct_.impls.push(generate_builder_pattern(
                pool,
                definition,
                message_name,
                slice,
            ));

            module.structs.push(struct_);
        }

        module.write_to(writer, 0)
    })
}

//fn generate_assert_impl<W: std::io::Write>(
//    writer: &mut W,
//    name: &str,
//) -> Result<(), std::io::Error> {
//    writeln!(writer, "    impl {name} {{")?;
//    writeln!(writer, "        const _ASSERT_CHECK_SIZE: () = {{")?;
//    writeln!(
//        writer,
//        "            assert!(size_of::<{name}>() == {MESSAGE_SIZE});"
//    )?;
//    writeln!(writer, "        }};")?;
//    writeln!(writer, "    }}\n")?;
//    Ok(())
//}

fn generate_builder_pattern<'a>(
    pool: &'a StringPool,
    definition: &ProtocolDefinition,
    message: &'a str,
    fields: &[FieldSymbol],
) -> Impl<'a> {
    let functions = fields
        .iter()
        .map(|f| {
            let ty = definition.table().get_type(f.type_id());
            let field_name = f.name().resolve(pool);
            Function {
                access: Access::Pub,
                is_const: true,
                name: Cow::Owned(format!("with_{field_name}")),
                self_pass: Some(Pass::Mut),
                arg: vec![Argument {
                    name: "value",
                    kind: ty.name().resolve(pool),
                    pass: Pass::Move,
                }],
                ret: Some("&mut Self"),
                body: Cow::Owned(format!("self.{field_name} = value; self")),
            }
        })
        .collect();
    Impl {
        target: message,
        functions,
    }
}

fn generate_message_trait_impl(message: &str) -> TraitImpl<'_> {
    TraitImpl {
        trait_name: TYPED_MESSAGE,
        target: message,
        functions: vec![
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("dst"),
                self_pass: Some(Pass::Ref),
                arg: vec![],
                ret: Some("u8"),
                body: Cow::Borrowed("self.dst"),
            },
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("with_dst"),
                self_pass: Some(Pass::Mut),
                arg: vec![Argument {
                    name: "dst",
                    kind: "u8",
                    pass: Pass::Move,
                }],
                ret: Some("&mut Self"),
                body: Cow::Borrowed("self.dst = dst; self"),
            },
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("src"),
                self_pass: Some(Pass::Ref),
                arg: vec![],
                ret: Some("u8"),
                body: Cow::Borrowed("self.src"),
            },
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("group"),
                self_pass: Some(Pass::Ref),
                arg: vec![],
                ret: Some("u8"),
                body: Cow::Borrowed("self.group"),
            },
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("with_group"),
                self_pass: Some(Pass::Mut),
                arg: vec![Argument {
                    name: "group",
                    kind: "u8",
                    pass: Pass::Move,
                }],
                ret: Some("&mut Self"),
                body: Cow::Borrowed("self.group = group; self"),
            },
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("kind"),
                self_pass: Some(Pass::Ref),
                arg: vec![],
                ret: Some("u8"),
                body: Cow::Borrowed("self.kind"),
            },
            Function {
                access: Access::None,
                is_const: false,
                name: Cow::Borrowed("with_kind"),
                self_pass: Some(Pass::Mut),
                arg: vec![Argument {
                    name: "kind",
                    kind: "u8",
                    pass: Pass::Move,
                }],
                ret: Some("&mut Self"),
                body: Cow::Borrowed("self.kind = kind; self"),
            },
        ],
    }
}

pub struct Module<'a> {
    access: Access,
    name: &'a str,
    enums: Vec<Enum<'a>>,
    structs: Vec<Struct<'a>>,
}

impl<'a> Module<'a> {
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self {
            access: Access::None,
            name,
            enums: vec![],
            structs: vec![],
        }
    }

    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "{} mod {} {{", self.access, self.name)?;

        for enum_ in &self.enums {
            enum_.write_to(writer, depth + 1)?;
        }

        for struct_ in &self.structs {
            struct_.write_to(writer, depth + 1)?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}

#[derive(Default)]
pub struct DeriveList<'a> {
    inner: Vec<&'a str>,
}

impl DeriveList<'_> {
    #[must_use]
    pub const fn new() -> Self {
        Self { inner: vec![] }
    }

    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        if self.inner.is_empty() {
            return Ok(());
        }

        add_depth(writer, depth)?;
        write!(writer, "#[derive(")?;

        for (idx, derive) in self.inner.iter().enumerate() {
            write!(writer, "{derive}")?;
            if idx < self.inner.len() - 1 {
                write!(writer, ", ")?;
            }
        }

        writeln!(writer, ")]")
    }
}

pub struct Enum<'a> {
    derives: DeriveList<'a>,
    access: Access,
    name: &'a str,
    variants: Vec<&'a str>,
}

impl<'a> Enum<'a> {
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self {
            derives: DeriveList::new(),
            access: Access::None,
            name,
            variants: vec![],
        }
    }

    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        self.derives.write_to(writer, depth)?;

        add_depth(writer, depth)?;
        writeln!(writer, "{} enum {} {{", self.access, self.name)?;

        for variant in &self.variants {
            add_depth(writer, depth + 1)?;
            writeln!(writer, "{variant},")?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}

pub struct Struct<'a> {
    add_repr: bool,
    derives: DeriveList<'a>,
    access: Access,
    name: &'a str,
    fields: Vec<Field<'a>>,
    trait_impls: Vec<TraitImpl<'a>>,
    impls: Vec<Impl<'a>>,
}

impl<'a> Struct<'a> {
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self {
            add_repr: false,
            derives: DeriveList::new(),
            access: Access::None,
            name,
            fields: vec![],
            trait_impls: vec![],
            impls: vec![],
        }
    }

    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        if self.add_repr {
            add_depth(writer, depth)?;
            writeln!(writer, "#[repr(C, align(32))]")?;
        }

        self.derives.write_to(writer, depth)?;

        add_depth(writer, depth)?;
        writeln!(writer, "{} struct {} {{", self.access, self.name)?;

        for field in &self.fields {
            field.write_to(writer, depth + 1)?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")?;

        for block in &self.trait_impls {
            block.write_to(writer, depth)?;
        }

        for block in &self.impls {
            block.write_to(writer, depth)?;
        }

        Ok(())
    }
}

pub struct Field<'a> {
    access: Access,
    name: Cow<'a, str>,
    kind: Cow<'a, str>,
    comment: Option<Cow<'a, str>>,
}

impl Field<'_> {
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        if let Some(comment) = self.comment.as_ref() {
            add_depth(writer, depth)?;
            writeln!(writer, "/* {comment} */")?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "{} {}: {},", self.access, self.name, self.kind)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Pass {
    Move,
    Ref,
    Mut,
}

impl Display for Pass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pass::Move => Ok(()),
            Pass::Ref => write!(f, "&"),
            Pass::Mut => write!(f, "&mut "),
        }
    }
}

pub enum Access {
    None,
    Pub,
}

impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Access::None => Ok(()),
            Access::Pub => write!(f, "pub"),
        }
    }
}

pub struct Function<'a> {
    access: Access,
    is_const: bool,
    name: Cow<'a, str>,
    self_pass: Option<Pass>,
    arg: Vec<Argument<'a>>,
    ret: Option<&'a str>,
    body: Cow<'a, str>,
}

fn add_depth<W: std::io::Write>(writer: &mut W, depth: usize) -> Result<(), std::io::Error> {
    for _ in 0..depth {
        write!(writer, "    ")?;
    }
    Ok(())
}

impl Function<'_> {
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        write!(writer, "{}", self.access)?;
        if self.is_const {
            write!(writer, " const")?;
        }
        write!(writer, " fn {}(", self.name)?;

        if let Some(pass) = self.self_pass {
            let arg = match pass {
                Pass::Move => "self",
                Pass::Ref => "&self",
                Pass::Mut => "&mut self",
            };
            write!(writer, "{arg}")?;
        }

        if !self.arg.is_empty() {
            write!(writer, ", ")?;
            for (idx, arg) in self.arg.iter().enumerate() {
                arg.write_to(writer)?;
                if idx != self.arg.len() - 1 {
                    write!(writer, ", ")?;
                }
            }
        }

        write!(writer, ")")?;

        if let Some(ret) = self.ret.as_ref() {
            write!(writer, " -> {ret}")?;
        }

        writeln!(writer, " {{")?;

        add_depth(writer, depth + 1)?;
        writeln!(writer, "{}", self.body)?;

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}

pub struct Argument<'a> {
    name: &'a str,
    kind: &'a str,
    pass: Pass,
}

impl Argument<'_> {
    fn write_to<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        write!(writer, "{}: {}{}", self.name, self.pass, self.kind)
    }
}

pub struct Impl<'a> {
    target: &'a str,
    functions: Vec<Function<'a>>,
}

impl Impl<'_> {
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "impl {} {{", self.target)?;

        for function in &self.functions {
            function.write_to(writer, depth + 1)?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}

pub struct TraitImpl<'a> {
    trait_name: &'a str,
    target: &'a str,
    functions: Vec<Function<'a>>,
}

impl TraitImpl<'_> {
    fn write_to<W: std::io::Write>(
        &self,
        writer: &mut W,
        depth: usize,
    ) -> Result<(), std::io::Error> {
        add_depth(writer, depth)?;
        writeln!(writer, "impl {} for {} {{", self.trait_name, self.target)?;

        for function in &self.functions {
            function.write_to(writer, depth + 1)?;
        }

        add_depth(writer, depth)?;
        writeln!(writer, "}}\n")
    }
}

#[cfg(test)]
mod tests {
    use creamy_utils::strpool::StringPool;
    use creamy_xmlc::ProtocolCompiler;

    use crate::{Args, generate};

    #[test]
    fn test() {
        let content = std::fs::read_to_string(
            "/mnt/ssd/fusionwm/creamy/devkit/creamy-xmlc/tests/success.xml",
        )
        .unwrap();
        let mut pool = StringPool::default();
        let mut compiler = ProtocolCompiler::new(&mut pool);
        let protocol = compiler.compile(&content).unwrap();

        let mut bytes = Vec::with_capacity(8192 * 2);
        generate(&mut bytes, Args::default(), &pool, &protocol).unwrap();
        let string = String::from_utf8(bytes).unwrap();
        println!("{string}");
    }
}
