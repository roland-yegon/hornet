use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::type_system::TypeSystem;
use serde_json::json;

pub struct StingerLsp {
    // State for the language server
}

impl StingerLsp {
    pub fn new() -> Self {
        StingerLsp {}
    }

    pub fn start(&self) {
        // In a real implementation, this would read from stdin/stdout
        // and handle JSON-RPC requests for Hover, Definition, and Diagnostics.
        println!("Stinger Language Server for Hornet is running...");
    }

    pub fn get_diagnostics(&self, source: &str) -> serde_json::Value {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        let mut checker = TypeChecker::new();
        
        match checker.check(&ast) {
            Ok(_) => json!([]),
            Err(msg) => json!([{
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 10 } },
                "severity": 1,
                "message": msg
            }]),
        }
    }
}
