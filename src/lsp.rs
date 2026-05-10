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
}

impl Default for StingerLsp {
    fn default() -> Self {
        Self::new()
    }
}

impl StingerLsp {
    pub fn start(&self) {
        // In a real implementation, this would read from stdin/stdout
        // and handle JSON-RPC requests for Hover, Definition, and Diagnostics.
        let _ = self.get_diagnostics("");
        println!("Stinger Language Server for Hornet is running...");
    }

    pub fn get_diagnostics(&self, source: &str) -> serde_json::Value {
        let mut lexer = Lexer::new(source);
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(err) => return json!([{
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 10 } },
                "severity": 1,
                "message": err.to_string()
            }]),
        };

        let mut parser = Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(err) => return json!([{
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 10 } },
                "severity": 1,
                "message": err.to_string()
            }]),
        };

        let mut checker = TypeSystem::new();
        match checker.analyze(&ast) {
            Ok(_) => json!([]),
            Err(msg) => json!([{
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 10 } },
                "severity": 1,
                "message": msg.to_string()
            }]),
        }
    }
}
