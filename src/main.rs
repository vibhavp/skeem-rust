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

    if cfg!(debug) {
                println!("skeem debug buil");
    }
    print!("LISP> ");
    io::stdout().flush().unwrap();
    loop {
        i.gc_disable();
        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        i.gc_disable();
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
