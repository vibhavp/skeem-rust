use error::Err;
use environment::Environment;

use std::collections::LinkedList;
use std::boxed::Box;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Add;
use std::ops::Mul;
use std::ops::Div;
use std::fmt;
use std::option::Option;

pub type HeapObject = Rc<RefCell<Box<Object>>>;
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
    Nil,
}

pub struct Object {
    pub object_type: Type,
    pub marked: bool,
}

pub struct Lambda {
    pub env: Option<Environment>, //type is environment
    pub params: HeapObject, //type is Cons
    pub body: HeapObject, //type is Cons
}

impl Lambda {
    fn mark(&mut self) {
        if let Some(ref mut env) = self.env {
            env.mark_all();
        }
        self.params.borrow_mut().mark();
        self.body.borrow_mut().mark();
    }
}

pub enum Procedure {
    Lambda (Lambda), //env type is Environment
    Primitive(&'static Fn(&List) -> Result<HeapObject, Err>)
}

impl Object {
    pub fn new(t: Type) -> Object {
        Object{object_type: t, marked: false}
    }

    #[inline(always)]
    pub fn unwrap_list(&self) -> &List {
        if let Type::Cons(ref l) = self.object_type {
            l
        } else {
            panic!("object is not a list")
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
            Type::Nil => "nil"
        }
    }

    pub fn mark(&mut self) {
        if self.marked {
            return
        }

        self.marked = true;
        match self.object_type {
            Type::Cons(ref mut cons) => Object::mark_list(cons),
            Type::Procedure(ref mut procedure) => Object::mark_procedure(procedure.as_mut()),
            _ => {},
        };
    }

    fn mark_procedure(procedure: &mut Procedure) {
        match procedure {
            &mut Procedure::Lambda(ref mut procedure) => {procedure.mark();},
            &mut Procedure::Primitive(_) => {},
        }
    }

    fn mark_list(cons: &mut List) {
        for obj in cons {
            obj.borrow_mut().mark();
        }
    }

    pub fn add_list(nums: &List) -> Result<Object, Err> {
        let mut sum = Object::new(Type::Integer(0));
        for obj in nums {
            match obj.borrow().object_type {
                Type::Float(n) => {sum = sum + Object::new(Type::Float(n))},
                Type::Integer(n) => {sum = sum + Object::new(Type::Integer(n))}
                _ => return Result::Err(Err::WrongType{wanted: "numberp", got: obj.borrow().get_type_string()})
            }
        }

        Result::Ok(sum)
    }

    pub fn sub_list(nums: &List) -> Result<Object, Err> {
        let mut sum = Object::new(Type::Integer(0));
        for obj in nums {
            match obj.borrow().object_type {
                Type::Float(n) => {sum = sum + Object::new(Type::Float(-n))},
                Type::Integer(n) => {sum = sum + Object::new(Type::Integer(-n))}
                _ => return Result::Err(Err::WrongType{wanted: "numberp", got: obj.borrow().get_type_string()})

           }
        }

        Result::Ok(sum)
    }

    pub fn mul_list(nums: &List) -> Result<Object, Err> {
        let mut prod = Object::new(Type::Integer(0));
        for obj in nums {
            match obj.borrow().object_type {
                Type::Float(n) => {prod = prod * Object::new(Type::Float(n))},
                Type::Integer(n) => {prod = prod * Object::new(Type::Integer(n))}
                _ => return Result::Err(Err::WrongType{wanted: "numberp", got: obj.borrow().get_type_string()})
            }
        }

        Result::Ok(prod)

    }

    pub fn div_list(nums: &List) -> Result<Object, Err> {
        let mut prod = Object::new(Type::Integer(0));
        for obj in nums {
            match obj.borrow().object_type {
                Type::Float(n) => {prod = prod / Object::new(Type::Float(n))},
                Type::Integer(n) => {prod = prod / Object::new(Type::Integer(n))}
                _ => return Result::Err(Err::WrongType{wanted: "numberp", got: obj.borrow().get_type_string()})
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
            Type::String(ref s) => write!(f, "{}", s),
            Type::Cons(ref l) => {
                for obj in l.iter() {
                    let res = write!(f, "{}", *obj.borrow());
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
            Type::Nil => write!(f, "nil"),
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
