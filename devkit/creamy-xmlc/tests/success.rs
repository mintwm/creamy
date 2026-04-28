use std::io::Cursor;

use binrw::{BinRead, BinWrite};
use creamy_utils::strpool::StringPool;
use creamy_xmlc::{Access, ProtocolCompiler, ProtocolDefinition, StringPoolResolver, Version};

const SUCCESS_TEST: &str = include_str!("success.xml");

#[test]
fn success() {
    let mut pool = StringPool::default();
    let mut compiler = ProtocolCompiler::new(&mut pool);
    let protocol = compiler.compile(SUCCESS_TEST).unwrap();
    assert_eq!(protocol.name().resolve(&pool), "success_test");
    assert_eq!(protocol.version(), Version { major: 0, minor: 1 });
    assert_eq!(protocol.access(), Access::MultipleWrite);
    assert_eq!(protocol.table().type_count(), 16); //Builtin (12) + Custom (4)
    let first = &protocol.table().types()[12];
    assert_eq!(first.name().resolve(&pool), "BucketSmall");

    let second = &protocol.table().types()[13];
    assert_eq!(second.name().resolve(&pool), "Status");

    for group in protocol.groups() {
        println!("-----  {}  -----", group.name().resolve(&pool));
        println!("structs: {}", group.structs());
        println!("enums: {}", group.enums());
    }

    for ty in protocol.table().types() {
        println!("Type: {}", ty.name().resolve(&pool));
    }
}

#[test]
fn serialize_and_deserialize() {
    let mut pool = StringPool::default();
    let mut compiler = ProtocolCompiler::new(&mut pool);

    let original = compiler
        .compile(SUCCESS_TEST)
        .expect("Failed to compile XML");

    let mut buffer = Vec::new();
    let mut writer = Cursor::new(&mut buffer);
    original.write_le(&mut writer).expect("Failed to serialize");

    let mut reader = Cursor::new(&buffer);
    let deserialized =
        ProtocolDefinition::read_le(&mut reader).expect("Failed to deserialize from binary");

    assert_eq!(
        original, deserialized,
        "Protocol changed after round-trip serialization"
    );

    assert_eq!(reader.position(), buffer.len() as u64);
}
