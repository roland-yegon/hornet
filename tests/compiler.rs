use hornet::codegen::Codegen;
use hornet::lexer::Lexer;
use hornet::parser::Parser;
use hornet::type_system::TypeSystem;

#[test]
fn compiler_pipeline_generates_llvm_ir() {
    let source = r#"
fn main():
    print("Hello Hornet")
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");

    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("parsing failed");

    let mut type_system = TypeSystem::new();
    type_system.analyze(&program).expect("type checking failed");

    let mut codegen = Codegen::new();
    let output = codegen.generate(&program);

    assert!(output.contains("declare i32 @printf(i8*, ...)"));
    assert!(output.contains("define i32 @main()"));
    assert!(output.contains("call i32 (i8*, ...) @printf"));
}

#[test]
fn type_system_rejects_invalid_if_conditions() {
    let source = r#"
if 1:
    print("bad")
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("parsing failed");
    let mut type_system = TypeSystem::new();
    assert!(type_system.analyze(&program).is_err());
}
