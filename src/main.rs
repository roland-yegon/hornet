mod lexer;
mod ast;
mod parser;

use std::env;
use std::fs;
use crate::lexer::Lexer;
use crate::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: hornet <command> <file>");
        println!("Commands: tokenize, parse");
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    let source = fs::read_to_string(filename).expect("Failed to read file");

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    match command.as_str() {
        "tokenize" => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        "parse" => {
            let mut parser = Parser::new(tokens);
            let ast = parser.parse();
            println!("{}", serde_json::to_string_pretty(&ast).unwrap());
        }
        _ => println!("Unknown command: {}", command),
    }
}
