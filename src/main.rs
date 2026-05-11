mod lexer;
mod ast;
mod parser;
mod error;
mod type_system;
mod coari;
mod codegen;
mod stdlib;
mod lsp;
mod interpreter;

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

fn print_help() {
    println!("Hornet Language Compiler v0.1.0");
    println!();
    println!("Usage: hornet <command> [<file>]");
    println!();
    println!("Commands:");
    println!("  tokenize <file>    Tokenize source file and display tokens");
    println!("  parse <file>       Parse source file and output AST as JSON");
    println!("  check <file>       Type-check program without compilation");
    println!("  build <file>       Compile to LLVM IR (.ll format)");
    println!("  run <file>         Execute program");
    println!("  lsp                Start language server protocol daemon");
    println!("  --help, -h         Show this help message");
    println!("  --version, -v      Show version information");
    println!();
    println!("Examples:");
    println!("  hornet tokenize hello.hn");
    println!("  hornet parse program.hn");
    println!("  hornet check program.hn");
    println!("  hornet build program.hn");
    println!("  hornet run program.hn");
    println!();
    println!("For more information, visit: https://github.com/roland-yegon/hornet");
}

fn main() -> Result<(), HornetError> {
    let args: Vec<String> = env::args().collect();
    
    // Handle --help, -h, --version, -v
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--version" | "-v" => {
                println!("hornet version 0.1.0");
                return Ok(());
            }
            _ => {}
        }
    }
    
    if args.len() < 3 && (args.len() < 2 || args[1] != "lsp") {
        println!("Usage: hornet <command> <file>");
        println!("Commands: tokenize, parse, check, build, run, lsp");
        println!("Use 'hornet --help' for detailed information");
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
            let ast = parser.parse()?;
            let mut checker = TypeSystem::new();
            checker.analyze(&ast)?;
            let mut interpreter = crate::interpreter::Interpreter::new();
            interpreter.run(&ast)?;
        }
        _ => println!("Unknown command: {}", command),
    }
    
    Ok(())
}
