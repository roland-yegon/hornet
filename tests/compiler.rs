use hornet::codegen::Codegen;
use hornet::lexer::Lexer;
use hornet::parser::Parser;
use hornet::type_system::TypeSystem;

#[test]
fn compiler_pipeline_generates_llvm_ir() {
    let source = r#"
define main():
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
fn codegen_supports_if_and_for_control_flow() {
    let source = r#"
for i from 1 to 3:
    if i is 1:
        print("first")
    else:
        print("other")
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("parsing failed");
    let mut type_system = TypeSystem::new();
    type_system.analyze(&program).expect("type checking failed");
    let mut codegen = Codegen::new();
    let output = codegen.generate(&program);
    assert!(output.contains("icmp eq"));
    assert!(output.contains("br i1"));
    assert!(output.contains("for.loop"));
}

#[test]
fn codegen_supports_record_member_access() {
    let source = r#"
record Point:
    x: Int
    y: Int

define main():
    p = Point(1, 2)
    print(p.x)
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("parsing failed");
    let mut type_system = TypeSystem::new();
    type_system.analyze(&program).expect("type checking failed");
    let mut codegen = Codegen::new();
    let output = codegen.generate(&program);
    assert!(output.contains("extractvalue %Point"));
    assert!(output.contains("call %Point @\"Point\"(i64 1, i64 2)"));
}

#[test]
fn type_system_rejects_record_constructor_with_wrong_field_type() {
    let source = r#"
record Point:
    x: Int
    y: Int

define main():
    p = Point(1, "two")
"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("parsing failed");
    let mut type_system = TypeSystem::new();
    assert!(type_system.analyze(&program).is_err());
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
