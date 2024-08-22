use std::env;
use std::fs;
use std::io::{self, Write};

use interpreter::Interpreter;
use parser::expression::Precedence;
use parser::Parser;
use token::Scanner;

pub(crate) mod interpreter;
pub(crate) mod parser;
pub(crate) mod token;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];
    let read_contents = || {
        let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
            writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
            String::new()
        });
        file_contents
    };

    match command.as_str() {
        "tokenize" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            // writeln!(io::stderr(), "Logs from your program will appear here!").unwrap();

            let mut found_lexical_error = false;
            let file_contents = read_contents();
            if !file_contents.is_empty() {
                let scanner = Scanner::new(file_contents);
                for token in scanner.iter() {
                    match token {
                        Ok(token) => println!("{:?}", token),
                        Err(token) => {
                            found_lexical_error = true;
                            eprintln!("{:?}", token)
                        }
                    }
                }
            } else {
                println!("EOF  null"); // Placeholder, remove this line when implementing the scanner
            }
            if found_lexical_error {
                std::process::exit(65);
            }
        }
        "parse" => {
            let file_contents = read_contents();
            let mut found_err = false;
            let mut parser = match Parser::from_source(file_contents) {
                Ok(parser) => parser,
                Err(e) => {
                    eprintln!("{:?}", e);
                    std::process::exit(65);
                }
            };

            // let mut parser = Parser::from_source(file_contents).unwrap();
            match parser.parse_expression(Precedence::Lowest) {
                Ok(expr) => println!("{:?}", expr),
                Err(e) => {
                    found_err = true;
                    eprintln!("{:?}", e);
                }
            };
            if found_err {
                std::process::exit(65);
            }
        }
        "evaluate" => {
            let file_contents = read_contents();
            let mut interpreter = match Interpreter::from_source(file_contents) {
                Ok(interpreter) => interpreter,
                Err(e) => {
                    eprintln!("{:?}", e);
                    std::process::exit(65)
                }
            };
            match interpreter.evaluate() {
                Ok(object) => println!("{}", object),
                Err(e) => {
                    eprintln!("{:?}", e);
                    std::process::exit(70);
                }
            }
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}
