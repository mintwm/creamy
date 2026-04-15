use creamy_compiler::ProtocolCompiler;

const SUCCESS_TEST: &str = include_str!("success.xml");

#[test]
fn success() {
    let mut compiler = ProtocolCompiler::new();
    compiler.compile(SUCCESS_TEST);
}
