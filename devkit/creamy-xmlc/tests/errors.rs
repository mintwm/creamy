mod generator;

use creamy_utils::strpool::StringPool;
use creamy_xmlc::{
    ProtocolCompiler, ProtocolDefinition,
    constraints::{MAX_ENUMS, MAX_GROUPS, MAX_MESSAGES_PER_GROUP, MAX_STRUCTS, MAX_VARIANTS},
    error::ProtocolError,
};

use crate::generator::XMLGeneratorBuilder;

fn get_xml(version: &str, content: &str) -> String {
    format!(
        r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol name="test" version="{version}" access="ExclusiveWrite">
    {content}
</protocol>"#
    )
}

fn compile(content: &str) -> Result<ProtocolDefinition, Vec<ProtocolError>> {
    let mut pool = StringPool::default();
    let mut compiler = ProtocolCompiler::new(&mut pool);
    compiler.compile(content)
}

#[test]
fn xml() {
    let result = compile(&get_xml("0.1", "<xml>"));
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap().first().unwrap(),
        ProtocolError::Xml(_)
    ));
}

#[test]
fn unknown_tag() {
    assert_eq!(
        compile(&get_xml("0.1", "<error></error>")),
        Err(vec![ProtocolError::UnknownTag("error".to_string())])
    );
}

#[test]
fn missing_protocol_token() {
    const CONTENT: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<enum name="missing">
</enum>
"#;
    assert_eq!(
        compile(CONTENT),
        Err(vec![ProtocolError::MissingProtocolToken])
    );
}

#[test]
fn double_remainder() {
    const CONTENT: &str = r#"
<group name="Group">
    <message name="Message">
        <field name="field" type="f32"/>
        <remainder name="data"/>
        <remainder name="data1"/>
    </message>
</group>
"#;
    assert_eq!(
        compile(&get_xml("1337.1337", CONTENT)),
        Err(vec![ProtocolError::UnexpectedToken(
            "<remainder>".to_string()
        )])
    );
}

#[test]
fn invalid_version_format() {
    assert_eq!(
        compile(&get_xml("0.0.0.0", "")),
        Err(vec![ProtocolError::InvalidVersionFormat])
    );
}

#[test]
fn invalid_minor() {
    assert_eq!(
        compile(&get_xml("0.65823", "")),
        Err(vec![ProtocolError::InvalidMinor])
    );

    assert_eq!(
        compile(&get_xml("0.-1024", "")),
        Err(vec![ProtocolError::InvalidMinor])
    );
}

#[test]
fn invalid_major() {
    assert_eq!(
        compile(&get_xml("69391.0", "")),
        Err(vec![ProtocolError::InvalidMajor])
    );

    assert_eq!(
        compile(&get_xml("-1.0", "")),
        Err(vec![ProtocolError::InvalidMajor])
    );
}

#[test]
fn unexpected_token() {
    let content = r#"
    <enum name="Mode">
        <field name="value" type="u128"/>
    </enum>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::UnexpectedToken("<field>".to_string())])
    );
}

#[test]
fn invalid_access() {
    const CONTENT: &str = r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol name="test" version="0.0" access="777"/>
"#;
    assert_eq!(
        compile(CONTENT),
        Err(vec![ProtocolError::InvalidAccess("777".to_string())])
    );
}

#[test]
fn too_many_groups_start() {
    let generator = XMLGeneratorBuilder::default()
        .groups(MAX_GROUPS + 100)
        .messages_per_group(1)
        .fields_per_message(1)
        .build();

    assert_eq!(
        compile(&get_xml("0.0", &generator.collect::<String>())),
        Err(vec![ProtocolError::TooManyGroups])
    );
}

#[test]
fn too_many_groups_end() {
    let generator = XMLGeneratorBuilder::default()
        .groups(MAX_GROUPS + 1)
        .messages_per_group(1)
        .fields_per_message(1)
        .build();

    assert_eq!(
        compile(&get_xml("0.0", &generator.collect::<String>())),
        Err(vec![ProtocolError::TooManyGroups])
    );
}

#[test]
fn message_too_many_fields() {
    let generator = XMLGeneratorBuilder::default()
        .groups(1)
        .messages_per_group(MAX_MESSAGES_PER_GROUP)
        .fields_per_message(28)
        .build();

    assert_eq!(
        compile(&get_xml("0.0", &generator.collect::<String>())),
        Err(vec![ProtocolError::TooManyFields])
    );
}

#[test]
fn structs_too_many_fields() {
    let generator = XMLGeneratorBuilder::default()
        .groups(1)
        .structs_per_group(MAX_MESSAGES_PER_GROUP)
        .fields_per_struct(28)
        .build();

    assert_eq!(
        compile(&get_xml("0.0", &generator.collect::<String>())),
        Err(vec![ProtocolError::TooManyFields])
    );
}

#[test]
fn too_many_messages() {
    let generator = XMLGeneratorBuilder::default()
        .groups(MAX_GROUPS)
        .messages_per_group(MAX_MESSAGES_PER_GROUP + 1) //TODO check
        .build();
    assert_eq!(
        compile(&get_xml("0.0", &generator.collect::<String>())),
        Err(vec![ProtocolError::TooManyMessages])
    );
}

#[test]
fn too_many_structs() {
    let generator = XMLGeneratorBuilder::default()
        .groups(1)
        .structs_per_group(MAX_STRUCTS + 1)
        .build();

    assert_eq!(
        compile(&get_xml("0.0", &generator.collect::<String>())),
        Err(vec![ProtocolError::TooManyStructs])
    );
}

#[test]
fn too_many_enums() {
    let generator = XMLGeneratorBuilder::default()
        .groups(1)
        .enums_per_group(MAX_ENUMS + 1)
        .build();

    assert_eq!(
        compile(&get_xml("0.0", &generator.collect::<String>())),
        Err(vec![ProtocolError::TooManyEnums])
    );
}

#[test]
fn too_many_variants() {
    let generator = XMLGeneratorBuilder::default()
        .groups(1)
        .enums_per_group(1)
        .variants_per_enum(MAX_VARIANTS + 1)
        .build();

    assert_eq!(
        compile(&get_xml("0.0", &generator.collect::<String>())),
        Err(vec![ProtocolError::TooManyVariants])
    );
}

fn assert_missing_name_attribute(content: &str, tag: &str, attr: &str) {
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::MissingAttribute {
            tag: tag.to_string(),
            attr: attr.to_string()
        }])
    );
}

#[test]
fn protocol_missing_name_attribute() {
    const CONTENT: &str = r"<protocol></protocol>";
    assert_missing_name_attribute(CONTENT, "protocol", "name");
}

#[test]
fn protocol_missing_version_attribute() {
    const CONTENT: &str = r#"<protocol name="test"></protocol>"#;
    assert_missing_name_attribute(CONTENT, "protocol", "version");
}

#[test]
fn protocol_missing_access_attribute() {
    const CONTENT: &str = r#"<protocol name="test" version="1.0"></protocol>"#;
    assert_missing_name_attribute(CONTENT, "protocol", "access");
}

#[test]
fn group_missing_name_attribute() {
    const CONTENT: &str = r"<group></group>";
    assert_missing_name_attribute(CONTENT, "group", "name");
}

#[test]
fn message_missing_name_attribute() {
    const CONTENT: &str = r"<message></message>";
    assert_missing_name_attribute(CONTENT, "message", "name");
}

#[test]
fn struct_missing_name_attribute() {
    const CONTENT: &str = r"<struct></struct>";
    assert_missing_name_attribute(CONTENT, "struct", "name");
}

#[test]
fn enum_missing_name_attribute() {
    const CONTENT: &str = r"<enum></enum>";
    assert_missing_name_attribute(CONTENT, "enum", "name");
}

#[test]
fn variant_missing_name_attribute() {
    const CONTENT: &str = r"<variant></variant>";
    assert_missing_name_attribute(CONTENT, "variant", "name");
}

#[test]
fn remainder_missing_name_attribute() {
    const CONTENT: &str = r"<remainder/>";
    assert_missing_name_attribute(CONTENT, "remainder", "name");
}

#[test]
fn field_missing_name_attribute() {
    const CONTENT: &str = r"<field></field>";
    assert_missing_name_attribute(CONTENT, "field", "name");
}

#[test]
fn field_missing_type_attribute() {
    const CONTENT: &str = r#"<field name="test"></field>"#;
    assert_missing_name_attribute(CONTENT, "field", "type");
}

#[test]
fn struct_size_limit_exceeded() {
    let content = r#"
    <group name="test">
        <struct name="SyncValue">
            <field name="value0" type="u128"/>
            <field name="value1" type="u128"/>
            <field name="value2" type="u128"/>
            <field name="value3" type="u128"/>
            <field name="value4" type="u128"/>
            <field name="value5" type="u128"/>
            <field name="value6" type="u128"/>
            <field name="value7" type="u128"/>
            <field name="value10" type="u128"/>
            <field name="value11" type="u128"/>
            <field name="value12" type="u128"/>
            <field name="value13" type="u128"/>
            <field name="value14" type="u128"/>
            <field name="value15" type="u128"/>
            <field name="value16" type="u128"/>
            <field name="value17" type="u128"/>
        </struct>
    </group>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::InvalidSize(260)])
    );
}

#[test]
fn message_size_limit_exceeded() {
    let content = r#"
    <group name="test">
        <message name="SyncValue">
            <field name="value0" type="u8"/>
            <field name="value1" type="u16"/>
            <field name="value2" type="u32"/>
            <field name="value3" type="u64"/>
            <field name="value4" type="u128"/>

            <field name="value5" type="i8"/>
            <field name="value6" type="i16"/>
            <field name="value7" type="i32"/>
            <field name="value8" type="i64"/>
            <field name="value9" type="i128"/>

            <field name="value10" type="f32"/>
            <field name="value11" type="f64"/>
        </message>
    </group>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::InvalidSize(84)])
    );
}

//TODO: validate identifiers
//TODO: enum
//TODO: missing group
#[test]
fn forbidden_size() {
    //TODO enum size
    let content = r#"
    <group name="test">
        <struct name="test0"/>
        <message name="test1"/>
        <enum name="test2"/>
    </group>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![
            ProtocolError::InvalidSize(0),
            ProtocolError::InvalidSize(0),
        ])
    );
}

#[test]
fn array_size_error() {
    let content = r#"
    <group name="test">
        <message name="test">
            <field name="field" type="[u8; should_be_a_number]"/>
        </message>
    </group>
"#;
    let result = compile(&get_xml("0.0", content));
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert_eq!(err.len(), 1);
    assert!(matches!(err[0], ProtocolError::IntParse(_)));
}

#[test]
fn array_syntax_0() {
    let content = r#"
    <group name="test">
        <message name="test">
            <field name="field" type="[u8;28"/>
        </message>
    </group>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::Syntax {
            src: "[u8;28".to_string(),
            should_be: "[TYPE; SIZE]".to_string()
        },])
    );
}

#[test]
fn array_syntax_1() {
    let content = r#"
    <group name="test">
        <message name="test">
            <field name="field" type="u8;28]"/>
        </message>
    </group>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::Syntax {
            src: "u8;28]".to_string(),
            should_be: "[TYPE; SIZE]".to_string()
        },])
    );
}

#[test]
fn array_syntax_2() {
    let content = r#"
    <group name="test">
        <struct name="test">
            <field name="field" type="[28]"/>
        </struct>
    </group>
"#;
    assert_eq!(
        compile(&get_xml("0.0", content)),
        Err(vec![ProtocolError::Syntax {
            src: "[28]".to_string(),
            should_be: "[TYPE; SIZE]".to_string()
        },])
    );
}
