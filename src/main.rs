mod lexer;
mod ast;
mod parser;
mod error;
mod type_system;
mod coari;
mod codegen;
mod driver;
mod stdlib;
mod lsp;
mod interpreter;

use std::env;
use std::fs;
use crate::error::HornetError;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::type_system::TypeSystem;
use crate::coari::CoariAnalyzer;
use crate::codegen::Codegen;
use crate::lsp::StingerLsp;

fn print_help() {
    println!("Hornet Language Compiler v0.2.0");
    println!();
    println!("Usage: hornet <command> [options] <file>");
    println!();
    println!("Commands:");
    println!("  tokenize <file>            Tokenize source file and display tokens");
    println!("  parse <file>               Parse source file and output AST as JSON");
    println!("  check <file>               Type-check program without compilation");
    println!("  build <file>               Compile to native binary");
    println!("  run <file>                 Execute program immediately using the interpreter");
    println!("  lsp                        Start language server protocol daemon");
    println!("  --help, -h                 Show this help message");
    println!("  --version, -v              Show version information");
    println!();
    println!("Build options:");
    println!("  --release                  Enable optimizations (-O2)");
    println!("  --emit-ir                  Also write LLVM IR to <file>.ll");
    println!();
    println!("Examples:");
    println!("  hornet tokenize hello.hn");
    println!("  hornet parse program.hn");
    println!("  hornet check program.hn");
    println!("  hornet build hello.hn");
    println!("  hornet build --release app.hn");
    println!("  hornet build --emit-ir app.hn");
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
                println!("hornet version 0.2.0");
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

    if command == "lsp" {
        let lsp = StingerLsp::new();
        lsp.start();
        return Ok(());
    }

    let filename_raw = if args.len() > 2 { &args[2] } else { "" };

    match command.as_str() {
        "tokenize" => {
            if filename_raw.is_empty() {
                return Err(HornetError::Other("No filename provided for tokenize".into()));
            }
            let source = fs::read_to_string(filename_raw)?;
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize()?;
            for token in tokens {
                println!("{:?}", token);
            }
        }
        "parse" => {
            if filename_raw.is_empty() {
                return Err(HornetError::Other("No filename provided for parse".into()));
            }
            let source = fs::read_to_string(filename_raw)?;
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize()?;
            let mut parser = Parser::new(tokens);
            let ast = parser.parse()?;
            println!("{}", serde_json::to_string_pretty(&ast)?);
        }
        "check" => {
            if filename_raw.is_empty() {
                return Err(HornetError::Other("No filename provided for check".into()));
            }
            let source = fs::read_to_string(filename_raw)?;
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize()?;
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
            let mut release = false;
            let mut emit_ir = false;
            let mut filename = None;
            for arg in &args[2..] {
                match arg.as_str() {
                    "--release" => release = true,
                    "--emit-ir" => emit_ir = true,
                    other => {
                        if filename.is_none() {
                            filename = Some(other.to_string());
                        } else {
                            return Err(HornetError::Other(format!("Unexpected build option: {}", other)));
                        }
                    }
                }
            }

            let filename = filename.ok_or_else(|| HornetError::Other("No file provided for build".into()))?;
            let source = fs::read_to_string(&filename)?;
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize()?;
            let mut parser = Parser::new(tokens);
            let ast = parser.parse()?;
            let mut checker = TypeSystem::new();
            checker.analyze(&ast)?;
            let mut coari = CoariAnalyzer::new();
            coari.analyze(&ast)?;
            let mut gen = Codegen::new();
            let ir = gen.generate(&ast);
            let binary = driver::build_native(&filename, &source, &ir, release, emit_ir)?;
            println!("Built: {}", binary);
        }
        "run" => {
            if filename_raw.is_empty() {
                return Err(HornetError::Other("No filename provided for run".into()));
            }
            let source = fs::read_to_string(filename_raw)?;
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize()?;
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
