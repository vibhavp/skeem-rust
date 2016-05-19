use error::Err;
use std::collections::{LinkedList, HashMap};
use std::boxed::Box;
use std::rc::Rc;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Div;
use std::fmt;
use std::option::Option;
use std::mem::size_of;
use std::cell::Cell;

pub type HeapObject = Rc<Box<Object>>;
pub type List = LinkedList<HeapObject>;

pub fn new_list() -> List {
    LinkedList::new()
}

pub enum Type {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Character(char),
    String(String),
    Symbol(String),

    Cons(Box<List>),
    Procedure(Box<Procedure>),
}

impl Type {
    pub fn size_of(&self) -> usize {
        match self {
            &Type::Bool(_) => size_of::<bool>(),
            &Type::Integer(_) => size_of::<i64>(),
            &Type::Float(_) => size_of::<f64>(),
            &Type::Character(_) => size_of::<char>(),
            &Type::String(ref s) | &Type::Symbol(ref s) => size_of::<u8>() * s.capacity(),
            &Type::Cons(_) => size_of::<List>(),
            &Type::Procedure(ref p) => {
                if let &Procedure::Lambda(_) = p.as_ref() {
                    size_of::<Lambda>()
                } else {0}
            }
        }
    }
}

pub struct Object {
    pub object_type: Type,
    pub marked: Cell<bool>,
}

// (lambda (a r g s) body)
pub struct Lambda {
    pub env: Option<Rc<HashMap<String, HeapObject>>>, //type is environment
    pub params: HeapObject, //type is Cons, represents (a r g s)
    pub body: HeapObject, //type is Cons, represents body
}

impl Lambda {
    fn mark(&self) {
        if let Some(ref env) = self.env {
            for (_, obj) in env.iter() {
                obj.mark();
            }
        }
        self.params.mark();
        self.body.mark();
    }
}

pub enum Procedure {
    Lambda (Lambda), //env type is Environment
    Primitive(&'static Fn(&List) -> Result<HeapObject, Err>)
}

impl Object {
    pub fn new(t: Type) -> Object {
        Object{object_type: t, marked: Cell::new(true)}
    }

    #[inline]
    pub fn unwrap_list(&self) -> &List {
        if let Type::Cons(ref l) = self.object_type {
            l
        } else {
            panic!("object is not a list")
        }
    }
    #[inline]
    pub fn unwrap_sym(&self) -> String {
        if let Type::String(ref s) = self.object_type {
            s.clone()
        } else {
            panic!("object is not a string")
        }
    }

    pub fn get_type_string(&self) -> &'static str {
        match self.object_type {
            Type::Bool(_) => "boolean",
            Type::Integer(_) => "integer",
            Type::Float(_) => "float",
            Type::Character(_) => "character",
            Type::String(_) => "string",
            Type::Cons(_) => "list",
            Type::Procedure(_) => "procedure",
            Type::Symbol(_) => "symbol",
        }
    }

    pub fn mark(&self) {
        if self.marked.get() {
            return
        }

        self.marked.set(true);
        match self.object_type {
            Type::Cons(ref cons) => Object::mark_list(cons),
            Type::Procedure(ref procedure) => Object::mark_procedure(procedure),
            _ => {},
        };
    }

    fn mark_procedure(procedure: &Procedure) {
        match procedure {
            &Procedure::Lambda(ref procedure) => {procedure.mark();},
            &Procedure::Primitive(_) => {},
        }
    }

    fn mark_list(cons: &List) {
        for obj in cons {
            obj.mark();
        }
    }

    pub fn add_list(nums: &List) -> Result<Object, Err> {
        let mut sum = Object::new(Type::Integer(0));
        for obj in nums {
            match obj.object_type {
                Type::Float(n) => {sum = sum + Object::new(Type::Float(n))},
                Type::Integer(n) => {sum = sum + Object::new(Type::Integer(n))}
                _ => return Result::Err(Err::WrongType{wanted: "numberp", got: obj.get_type_string()})
            }
        }

        Result::Ok(sum)
    }

    pub fn sub_list(nums: &List) -> Result<Object, Err> {
        let mut sum = Object::new(Type::Integer(0));
        for obj in nums {
            match obj.as_ref().object_type {
                Type::Float(n) => {sum = sum + Object::new(Type::Float(-n))},
                Type::Integer(n) => {sum = sum + Object::new(Type::Integer(-n))}
                _ => return Result::Err(Err::WrongType{wanted: "numberp", got: obj.get_type_string()})

           }
        }

        Result::Ok(sum)
    }

    pub fn mul_list(nums: &List) -> Result<Object, Err> {
        let mut prod = Object::new(Type::Integer(0));
        for obj in nums {
            match obj.object_type {
                Type::Float(n) => {prod = prod * Object::new(Type::Float(n))},
                Type::Integer(n) => {prod = prod * Object::new(Type::Integer(n))}
                _ => return Result::Err(Err::WrongType{wanted: "numberp", got: obj.get_type_string()})
            }
        }

        Result::Ok(prod)
    }

    pub fn div_list(nums: &List) -> Result<Object, Err> {
        let mut prod = Object::new(Type::Integer(0));
        for obj in nums {
            match obj.object_type {
                Type::Float(n) => {prod = prod / Object::new(Type::Float(n))},
                Type::Integer(n) => {prod = prod / Object::new(Type::Integer(n))}
                _ => return Result::Err(Err::WrongType{wanted: "numberp", got: obj.get_type_string()})
            }
        }

        Result::Ok(prod)
    }
}

impl Add for Object {
    type Output = Object;

    fn add(self, other: Object) -> Object {
        match self.object_type {
            Type::Integer(n1) => match other.object_type {
                Type::Integer(n2) => (Object::new(Type::Integer(n1+n2))),
                Type::Float(n2) => (Object::new(Type::Float(n1 as f64+n2))),
                _ => panic!("n2 is not a number")
            },

            Type::Float(n1) => match other.object_type {
                Type::Integer(n2) => Object::new(Type::Float(n1+n2 as f64)),
                Type::Float(n2) => Object::new(Type::Float(n1+n2)),
                _ => panic!("n2 is not a number")
            },

            _ => panic!("n1 is not a number")
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(),fmt::Error> {
        match self.object_type {
            Type::Bool(b) => write!(f, "{}", b),
            Type::Integer(n) => write!(f,"{}", n),
            Type::Float(n) => write!(f, "{}", n),
            Type::Character(c) => write!(f, "?{}", c),
            Type::String(ref s) => write!(f, "\"{}\"", s),
            Type::Cons(ref l) => {
                if l.len() == 0 {
                    return write!(f, "nil");
                }
                for obj in l.iter() {
                    let res = write!(f, "{}", *obj.as_ref());
                    match res {
                        Ok(_) => {},
                        Err(e) => return Result::Err(e),
                    }
                };
                Result::Ok(())
            },
            Type::Procedure(_) => {
                write!(f, "procedure")
            },
            Type::Symbol(_) => panic!("write! used on symbol")
        }
    }
}

impl Mul for Object {
    type Output = Object;

    fn mul(self, other: Object) -> Object {
        match self.object_type {
            Type::Integer(n1) => match other.object_type {
                Type::Integer(n2) => (Object::new(Type::Integer(n1*n2))),
                Type::Float(n2) => (Object::new(Type::Float(n1 as f64*n2))),
                _ => panic!("n2 is not a number")
            },

            Type::Float(n1) => match other.object_type {
                Type::Integer(n2) => Object::new(Type::Float(n1*n2 as f64)),
                Type::Float(n2) => Object::new(Type::Float(n1*n2)),
                _ => panic!("n2 is not a number")
            },

            _ => panic!("n1 is not a number")
        }
    }
}

impl Div for Object {
    type Output = Object;

    fn div(self, other: Object) -> Object {
        match self.object_type {
            Type::Integer(n1) => match other.object_type {
                Type::Integer(n2) => (Object::new(Type::Integer(n1/n2))),
                Type::Float(n2) => (Object::new(Type::Float(n1 as f64/n2))),
                _ => panic!("n2 is not a number")
            },

            Type::Float(n1) => match other.object_type {
                Type::Integer(n2) => Object::new(Type::Float(n1/n2 as f64)),
                Type::Float(n2) => Object::new(Type::Float(n1/n2)),
                _ => panic!("n2 is not a number")
            },

            _ => panic!("n1 is not a number")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ops() {
        let mut o3 = Object::new(Type::Integer(1)) + Object::new(Type::Float(2.0));
        match o3.object_type {
            Type::Float(n) => {assert_eq!(n, 3.0)},
            _ => panic!("o3 should be a float")
        };

        let mut o3 = Object::new(Type::Integer(2)) * Object::new(Type::Float(2.0));
        match o3.object_type {
            Type::Float(n) => {assert_eq!(n, 4.0)},
            _ => panic!("o3 should be a float")
        };
    }
}
