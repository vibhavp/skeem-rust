use std::fmt;
use std::rc::Rc;

pub enum ErrType {
    WrongType{wanted: &'static str, got: &'static str},
    WrongArgsNum{wanted: usize, got: usize},
    WrongMinArgsNum{min: usize, got: usize},
    NotCallable(&'static str),
    SymbolNotFound(Rc<String>),
}

pub struct Err {
    err_type: ErrType,
    trace: Vec<Rc<String>>
}

impl Err {
    pub fn new(err_type: ErrType, trace: Vec<Rc<String>>) -> Err {
        Err{err_type: err_type, trace: trace}
    }
}

impl fmt::Display for Err {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "in function: \n");
        if self.trace.len() != 0 {
            for (i, fn_name) in (&self.trace).into_iter().enumerate() {
                let res = write!(f, "{}: {}\n", i, fn_name);
                if let Result::Err(e) = res {
                    return Result::Err(e);
                }
            }
        }
        write!(f, "{}", self.err_type)
    }
}

impl fmt::Display for ErrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrType::WrongType{wanted: w, got: g} =>write!(
                    f,"Wrong argument type, wanted: {}, got: {}", w, g),
            ErrType::WrongArgsNum{wanted: w, got: g} => write!(
                f, "Wrong number of arguments, wanted: {}, got: {}", w, g),
            ErrType::WrongMinArgsNum{min: m, got: g} => write!(
                f, "Wanted minimum {} args, got: {}", m, g
            ),
            ErrType::SymbolNotFound(ref sym) => write!(f, "Couldn't find symbol {}", sym),
            ErrType::NotCallable(t) => write!(f, "Type {} is not callable", t)
        }
    }
}


impl fmt::Debug for Err {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Debug for ErrType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
