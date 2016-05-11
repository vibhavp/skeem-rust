use std::fmt;

pub enum Err {
    WrongType{wanted: &'static str, got: &'static str},
    WrongArgsNum{wanted: usize, got: usize},
    NotCallable(&'static str),
    SymbolNotFound(String),
    EmptyList
}

impl fmt::Display for Err {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Err::WrongType{wanted: w, got: g} => write!(f, "Wrong argument type, wanted: {}, got: {}", w, g),
            Err::WrongArgsNum{wanted: w, got: g} => write!(f, "Wrong number of arguments, wanted: {}, got: {}", w, g),
            Err::SymbolNotFound(ref sym) => write!(f, "Couldn't find symbol {}", sym),
            Err::EmptyList => write!(f, "Cannot evaluate empty list"),
            Err::NotCallable(t) => write!(f, "Type {} is not callable", t)
        }
    }
}
