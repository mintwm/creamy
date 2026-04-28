use creamy_utils::strpool::StringPool;
use creamy_xmlc::{ProtocolCompiler, ProtocolDefinition, StringPoolResolver, error::ProtocolError};

use crate::generator::XMLGeneratorBuilder;

mod generator;

fn get_xml(content: &str) -> String {
    format!(
        r#"
<?xml version="1.0" encoding="UTF-8" ?>
<protocol name="test" version="1.0" access="ExclusiveWrite">
    {content}
</protocol>"#
    )
}

fn compile(pool: &mut StringPool, content: &str) -> Result<ProtocolDefinition, Vec<ProtocolError>> {
    let mut compiler = ProtocolCompiler::new(pool);
    compiler.compile(content)
}

#[test]
fn message_iterator() {
    let generator = XMLGeneratorBuilder::default()
        .groups(10)
        .messages_per_group(10)
        .fields_per_message(2)
        .build();

    let mut pool = StringPool::default();
    let protocol = compile(&mut pool, &get_xml(&generator.collect::<String>())).unwrap();
    let mut iterations = 0;
    let mut g_temp = 1;
    let mut m_temp = 0;
    protocol.message_iter(|g, m| {
        assert_eq!(m.name().resolve(&pool), format!("Message{m_temp}"));
        if m_temp == 10 {
            assert_eq!(g.name().resolve(&pool), format!("Group{g_temp}"));
            g_temp += 1;
        }
        m_temp += 1;
        iterations += 1;
    });

    assert_eq!(iterations, 10 * 10);
}
