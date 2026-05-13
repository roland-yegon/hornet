use hornet::ast::Stmt;
use hornet::lexer::Lexer;
use hornet::parser::Parser;

#[test]
fn define_keyword_parses_as_function() {
    let source = "define add(a, b):\n    return a + b\n";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parsing failed");
    assert!(matches!(ast.statements.get(0), Some(Stmt::FunctionDef { name, .. }) if name == "add"));
}

#[test]
fn for_from_to_syntax_parses_correctly() {
    let source = "for i from 1 to 5:\n    print(i)\n";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parsing failed");
    assert!(matches!(ast.statements.get(0), Some(Stmt::For { iterator, .. }) if iterator == "i"));
}

#[test]
fn for_from_upto_is_exclusive() {
    let source = "for i from 1 upto 3:\n    print(i)\n";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parsing failed");
    if let Some(Stmt::For { iterable, .. }) = ast.statements.get(0) {
        assert!(matches!(iterable, hornet::ast::Expr::Range { inclusive: false, .. }));
    } else {
        panic!("Expected for statement");
    }
}

#[test]
fn is_keyword_compiles_to_equality() {
    let source = "if x is 5:\n    print(x)\n";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parsing failed");
    if let Some(Stmt::If { condition, .. }) = ast.statements.get(0) {
        if let hornet::ast::Expr::BinaryOp { op, .. } = condition {
            assert_eq!(op, "==");
            return;
        }
    }
    panic!("Expected equality comparison");
}

#[test]
fn check_when_otherwise_parses() {
    let source = "check x:\n    when 1:\n        print(\"one\")\n    otherwise:\n        print(\"other\")\n";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parsing failed");
    assert!(matches!(ast.statements.get(0), Some(Stmt::Match { .. })));
}

#[test]
fn record_keyword_parses_as_record_def() {
    let source = "record Point:\n    x: Int\n    y: Int\n";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("tokenization failed");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parsing failed");
    assert!(matches!(ast.statements.get(0), Some(Stmt::RecordDef { name, .. }) if name == "Point"));
}
