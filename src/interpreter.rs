use types::{Object, Type, HeapObject, Lambda, Procedure, List};
use error::Err;
use environment::Environment;

use std::option::Option;
use std::result::Result;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Interpreter {
    live_objects: Vec<HeapObject>,
    environment: Environment,
    nil: HeapObject,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter{
            live_objects: Vec::new(),
            environment: Environment::new(),
            nil: Rc::new(RefCell::new(Box::new(Object::new(Type::Nil))))
        }
    }

    pub fn new_object(&mut self, t: Type) -> HeapObject {
        let obj = Rc::new(RefCell::new(Box::new(Object::new(t))));
        self.live_objects.push(obj.clone());
        obj
    }

    fn gc(&mut self) -> u64 {
        let mut count = 0 as u64;
        self.environment.mark_all();

        for i in 0..self.live_objects.len() {
            if !self.live_objects[i].borrow().marked {
                self.live_objects.swap_remove(i);
                count += 1;
                continue;
            }

            self.live_objects[i].borrow_mut().marked = false;
        }

        count
    }

    fn eval_lambda(&mut self, lambda: &Lambda, exp: List) -> Result<HeapObject, Err> {
        let params = lambda.params.borrow().unwrap_list().clone();
        if params.len() != exp.len() - 1 {
            return Result::Err(Err::WrongArgsNum{wanted: params.len(), got: exp.len()-1});
        }

        let mut closure = false;

        let mut last = self.nil.clone();

        // (lambda (a r g s) body)
        for obj in exp.iter().skip(2) {
            let res = self.eval(obj.clone());
            match res {
                Result::Ok(obj) => last = obj,
                Result::Err(err) => {
                    if closure {
                        self.environment.pop();
                    };
                    return Result::Err(err)
                }
            }
        }

        if closure {
            self.environment.pop();
        }

        Result::Ok(last)
    }

    fn eval_cons(&mut self, c: &List) -> Result<HeapObject, Err> {
        let frontopt = c.front();

        if let Option::None = frontopt {
            return Result::Err(Err::EmptyList);
        }

        let front = self.eval(frontopt.unwrap().clone());

        match front {
            Result::Ok(frontval) => {
                match frontval.borrow().object_type {
                    Type::Procedure(ref p) => match p.as_ref() {
                        &Procedure::Primitive(prim) => prim(c),
                        &Procedure::Lambda(ref lambda) => self.eval_lambda(lambda, c.clone())
                    },
                    _ => Result::Err(Err::NotCallable(frontval.borrow().get_type_string()))
                }
            },
            Result::Err(e) => Result::Err(e)
        }
    }

    pub fn eval(&mut self, hobj: HeapObject) -> Result<HeapObject, Err> {
        match hobj.borrow().object_type {
            Type::Cons(ref c) => self.eval_cons(c.as_ref()),
            Type::Symbol(ref sym) => {
                let val = try!(self.environment.find_sym(sym.clone()));
                Result::Ok(val.clone())
            },
            _ => Result::Ok(hobj.clone()),
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use types::Type;
    use std::string::ToString;

    #[test]
    fn test_gc() {
        let mut interpreter = Interpreter::new();

        let obj = interpreter.new_object(Type::String("foobar".to_string()));
        interpreter.environment.push();
        interpreter.environment.insert_sym("test".to_string(), obj.clone());
        assert_eq!(interpreter.gc(), 0);
        interpreter.environment.pop();
        assert_eq!(interpreter.gc(), 1);
        
        interpreter.new_object(Type::String("foobar".to_string()));
        assert_eq!(interpreter.gc(), 1);
        assert_eq!(interpreter.gc(), 0);
       
    }
}
