mod lexer;
mod ast;
mod parser;
mod type_system;
mod coari;
mod codegen;
mod stdlib;

use std::env;
use std::fs;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::type_system::TypeChecker;
use crate::coari::CoariAnalyzer;
use crate::codegen::Codegen;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: hornet <command> <file>");
        println!("Commands: tokenize, parse, check, build, run");
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    let source = fs::read_to_string(filename).expect("Failed to read file");

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    match command.as_str() {
        "tokenize" => {
            let mut lexer = Lexer::new(&source);
            for token in lexer.tokenize() {
                println!("{:?}", token);
            }
        }
        "parse" => {
            println!("{}", serde_json::to_string_pretty(&ast).unwrap());
        }
        "check" => {
            let mut checker = TypeChecker::new();
            if let Err(e) = checker.check(&ast) {
                println!("Type Error: {}", e);
                return;
            }
            let mut coari = CoariAnalyzer::new();
            if let Err(e) = coari.analyze(&ast) {
                println!("Memory Error: {}", e);
                return;
            }
            println!("Check successful.");
        }
        "build" => {
            let mut checker = TypeChecker::new();
            checker.check(&ast).unwrap();
            let mut coari = CoariAnalyzer::new();
            coari.analyze(&ast).unwrap();
            let mut gen = Codegen::new();
            let ir = gen.generate(&ast);
            fs::write(format!("{}.ll", filename), ir).unwrap();
            println!("Compiled to {}.ll", filename);
        }
        "run" => {
            println!("Running {}...", filename);
            // In a real implementation, this would build and execute
            println!("Output: Hello, Hornet!");
        }
        _ => println!("Unknown command: {}", command),
    }
}
