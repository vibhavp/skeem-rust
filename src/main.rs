extern crate skeem;

use skeem::interpreter::Interpreter;
use skeem::parse::{Scanner, parse_sexp};
use skeem::error::Err;
use skeem::types::HeapObject;
use std::io;
use std::io::Write;
use std::string::String;
use std::result::Result;

fn print_result(res: Result<HeapObject, Err>) {
    match res {
        Result::Ok(obj) => println!("=> {}", *(obj.as_ref())),
        Result::Err(err) => println!("error: {}", err),
    }
}

fn main() {
    let mut i = Interpreter::new();
    let mut scanner = Scanner::new();
    let stdin = io::stdin();

    print!("LISP> ");
    io::stdout().flush().unwrap();
    loop {
        i.gc_disable();
        let mut line = String::new();
        if let Result::Err(e) = stdin.read_line(&mut line) {
            println!("{}", e);
            return;
        };

        if line.chars().nth(0).unwrap() == '\n' {
            print!("LISP> ");
            io::stdout().flush().unwrap();
            continue;
        }
        let opt = scanner.scan(line);

        if let Option::Some(res) = opt {
            match res {
                Result::Ok(tokens) => {
                    let res = parse_sexp(tokens.as_ref(), &mut i);
                    match res {
                        Result::Ok(obj) => {i.gc_enable(); print_result(i.eval(obj))},
                        Result::Err(err) => println!("error: {}", err),
                    }
                },
                Result::Err(err) => {
                    println!("error: {}", err);
                }
            }
            print!("LISP> ");
            io::stdout().flush().unwrap();
        } else {
            print!("> ");
            io::stdout().flush().unwrap();
        }
    }
}
