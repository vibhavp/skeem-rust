use std::result::Result;
use std::boxed::Box;
use std::vec::Vec;
use std::clone::Clone;
use std::option::Option;
use std::fmt;
use std::str::FromStr;
use interpreter::Interpreter;
use types::{Type, new_list, HeapObject};

pub enum Token {
    ParenOpen,
    ParenClose,

    Symbol(String),
    String(String),
    Character(char),
    Integer(i64),
    Float(f64),
}

impl Clone for Token {
    fn clone(&self) -> Token {
        match self {
            &Token::ParenOpen => Token::ParenOpen,
            &Token::ParenClose => Token::ParenClose,
            &Token::String(ref s) => Token::String(s.clone()),
            &Token::Symbol(ref s) => Token::Symbol(s.clone()),
            &Token::Character(c) => Token::Character(c),
            &Token::Integer(i) => Token::Integer(i),
            &Token::Float(f) => Token::Float(f),
        }
    }
}

pub enum ScanError {
    UnmatchedParen,
    InvalidChar,
}

pub struct Scanner {
    scanning_string: bool,
    scanning_char: bool,
    scanning_num: bool,
    scanning_float: bool,
    scanning_list_depth: usize,
    tokens: Vec<Token>,
}

#[inline(always)]
fn is_terminating_char(ch: char) -> bool {
    ch == ' ' || ch == '(' || ch == ')'
}

impl Scanner{
    pub fn new() -> Scanner {
        Scanner {
            scanning_string: false,
            scanning_char: false,
            scanning_num: false,
            scanning_float: false,
            scanning_list_depth: 0,
            tokens: Vec::new(),
        }
    }

    #[inline(always)]
    fn scanning(&self) -> bool {
        self.scanning_char || self.scanning_list_depth != 0 || self.scanning_string
    }

    fn get_token(&self, word: &String) -> Option<Token> {
        if word.len() == 0 {
            return Option::None
        }

        if self.scanning_num {
            if self.scanning_float {
                let f = f64::from_str(word.clone().as_str()).unwrap();
                return Option::Some(Token::Float(f));
            }

            let n = i64::from_str(word.clone().as_str()).unwrap();
            return Option::Some(Token::Integer(n));
        }
        return Option::Some(Token::Symbol(word.clone()))
    }

    //Option::Some represents a completed scan
    //Option::None represents an incomplete scan
    pub fn scan(&mut self, line: String) -> Option<Result<Box<Vec<Token>>, ScanError>> {
        let mut tokens = if self.scanning() {self.tokens.clone()} else {Vec::new()};
        let mut word = String::new();
        let chars = line.chars();

        for (i, ch) in chars.enumerate() {
            let mut push_ch = false;

            match ch {
                '\"' => {
                    if self.scanning_string {
                        tokens.push(Token::String(word.clone()));
                        word.clear();
                    }

                    self.scanning_string = !self.scanning_string;
                },
                '?' => self.scanning_char = true,
                '0'...'9' => {self.scanning_num |= word.len() == 0; push_ch = true;},
                '.' => {self.scanning_float = self.scanning_num; push_ch = true;},
                '-' | '+' => {self.scanning_num |= word.len() == 0; push_ch = true;},
                '(' => {
                    self.get_token(&word).map(|t| {tokens.push(t)});
                    tokens.push(Token::ParenOpen);
                    self.scanning_list_depth += 1;
                    word.clear();
                },
                ')' => {
                    if !self.scanning_list_depth == 0 {
                        return Option::Some(Result::Err(ScanError::UnmatchedParen));
                    }
                    self.get_token(&word).map(|t| {tokens.push(t)});
                    tokens.push(Token::ParenClose);
                    self.scanning_list_depth -= 1;
                    word.clear();
                },
                ' ' => {
                    self.get_token(&word).map(|t| {tokens.push(t);});
                    word.clear();
                    self.scanning_num = false;
                },
                _ => {
                    if self.scanning_char {
                        if i < line.len() - 1 && !is_terminating_char(line.char_at(i+1)) {
                            return Option::Some(Result::Err(ScanError::InvalidChar));
                        }

                        self.scanning_char = false;
                        tokens.push(Token::Character(ch));
                        continue;
                    }
                    self.scanning_num = false;
                    push_ch = true;
                },
            };

            if push_ch {
                word.push(ch);
            }
        }

        //flush last token
        self.get_token(&word).map(|t| {tokens.push(t)});

        if self.scanning() {
            self.tokens = tokens;
            Option::None
        } else {
            Option::Some(Result::Ok(Box::new(tokens)))
        }
    }
}

fn parse_list(tokens: &Vec<Token>, start: usize, interpreter: &mut Interpreter) -> Option<HeapObject> {
    let mut list = Box::new(new_list());
    for (i, token) in tokens.into_iter().skip(start).enumerate() {
        match token {
            &Token::ParenOpen => {
                let obj = parse_list(tokens, i, interpreter);
                match obj {
                    Option::Some(hobj) => list.as_mut().push_back(hobj),
                    Option::None => return Option::Some(interpreter.new_object(Type::Cons(list))),
                }
            },
            &Token::ParenClose => {
                if list.len() == 0 {
                    return Option::Some(interpreter.new_nil());
                } else {
                    return Option::None;
                }
            },
            _ => list.as_mut().push_back(parse(token, interpreter)),
        }
    }

    panic!("unreachable")
}

fn parse(token: &Token, interpreter: &mut Interpreter) -> HeapObject {
    match token {
        &Token::Symbol(ref s) => interpreter.new_object(Type::Symbol(s.clone())),
        &Token::String(ref s) => interpreter.new_object(Type::String(s.clone())),
        &Token::Character(c) => interpreter.new_object(Type::Character(c)),
        &Token::Integer(i) => interpreter.new_object(Type::Integer(i)),
        &Token::Float(f) => interpreter.new_object(Type::Float(f)),
        &Token::ParenOpen | &Token::ParenClose => panic!("cannot parse parens")
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Token::ParenOpen => write!(f, "("),
            &Token::ParenClose => write!(f, ")"),
            &Token::Symbol(ref s) => write!(f, "[sym {}]", s),
            &Token::String(ref s) => write!(f,"\"{}\"", s),
            &Token::Character(c) => write!(f, "?{}", c),
            &Token::Integer(i) => write!(f, "[i {}]", i),
            &Token::Float(fl) => write!(f, "[f {}]", fl),
        }
   }
}

impl fmt::Debug for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scan() {
        let mut s = Scanner::new();
        let res = s.scan("(\"hi\" ?c 1 1.2 ())".to_string());
        let vec = res.unwrap().unwrap();
        for tok in vec.into_iter() {
            print!("{} ", tok);
        }
        println!("");
    }

    #[test]
    fn test_scan_err() {
        let mut s = Scanner::new();
        let res = s.scan("?abcd".to_string()).unwrap();
        match res {
            Result::Ok(_) => panic!("should error"),
            Result::Err(e) => {
                if let ScanError::InvalidChar = e {

                } else {
                    panic!("should be invalidchar")
                }
            },
        }
    }
}
