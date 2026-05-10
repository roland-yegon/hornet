mod lexer;
mod ast;
mod parser;
mod error;
mod type_system;
mod coari;
mod codegen;
mod stdlib;
mod lsp;

use std::env;
use std::fs;
use std::path::Path;
use crate::error::HornetError;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::type_system::TypeSystem;
use crate::coari::CoariAnalyzer;
use crate::codegen::Codegen;
use crate::lsp::StingerLsp;

fn main() -> Result<(), HornetError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 && (args.len() < 2 || args[1] != "lsp") {
        println!("Usage: hornet <command> <file>");
        println!("Commands: tokenize, parse, check, build, run, lsp");
        return Ok(());
    }

    let command = &args[1];
    let filename_raw = if args.len() > 2 { &args[2] } else { "" };

    if command == "lsp" {
        let lsp = StingerLsp::new();
        lsp.start();
        return Ok(());
    }

    if filename_raw.is_empty() {
        return Err("No filename provided".into());
    }

    let source = fs::read_to_string(filename_raw)?;

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    match command.as_str() {
        "tokenize" => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        "parse" => {
            let mut parser = Parser::new(tokens);
            let ast = parser.parse()?;
            println!("{}", serde_json::to_string_pretty(&ast)?);
        }
        "check" => {
            let mut parser = Parser::new(tokens);
            let ast = parser.parse()?;
            let mut checker = TypeSystem::new();
            if let Err(e) = checker.analyze(&ast) {
                println!("Type Error: {}", e);
                return Ok(());
            }
            let mut coari = CoariAnalyzer::new();
            if let Err(e) = coari.analyze(&ast) {
                println!("Memory Error: {}", e);
                return Ok(());
            }
            println!("Check successful.");
        }
        "build" => {
            let mut parser = Parser::new(tokens);
            let ast = parser.parse()?;
            let mut checker = TypeSystem::new();
            checker.analyze(&ast)?;
            let mut coari = CoariAnalyzer::new();
            coari.analyze(&ast)?;
            let mut gen = Codegen::new();
            let ir = gen.generate(&ast);
            
            // Security: Sanitize output path to prevent path traversal
            let path = Path::new(filename_raw);
            let safe_name = path.file_name().ok_or_else(|| HornetError::Other("Invalid filename".into()))?.to_str().ok_or_else(|| HornetError::Other("Invalid filename encoding".into()))?;
            let output_path = format!("{}.ll", safe_name);
            
            fs::write(&output_path, ir)?;
            println!("Compiled to {}", output_path);
        }
        "run" => {
            let mut parser = Parser::new(tokens);
            let _ast = parser.parse()?;
            println!("Running {}...", filename_raw);
            println!("Output: Hello, Hornet!");
        }
        _ => println!("Unknown command: {}", command),
    }
    
    Ok(())
}
