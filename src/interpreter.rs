use types::{Object, Type, HeapObject, Lambda, Procedure, List, new_list};
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
    bool_true: HeapObject,
    bool_false: HeapObject,
    gc_disabled: bool,
    bytes_alloc: usize,
    gc_threshold: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter{
            live_objects: Vec::new(),
            environment: Environment::new(),
            nil: Rc::new(RefCell::new(Box::new(Object::new(Type::Cons(Box::new(new_list())))))),
            bool_true: Rc::new(RefCell::new(Box::new(Object::new(Type::Bool(true))))),
            bool_false: Rc::new(RefCell::new(Box::new(Object::new(Type::Bool(false))))),
            gc_disabled: false,
            bytes_alloc: 0,
            gc_threshold: 0,
        }
    }

    #[inline]
    pub fn new_nil(&self) -> HeapObject {self.nil.clone()}
    #[inline]
    pub fn new_true(&self) -> HeapObject {self.bool_true.clone()}
    #[inline]
    pub fn new_false(&self) -> HeapObject {self.bool_false.clone()}

    pub fn new_object(&mut self, t: Type) -> HeapObject {
        self.bytes_alloc += t.size_of();
        if self.bytes_alloc > self.gc_threshold {
            self.gc_threshold = self.bytes_alloc/2;
            let n = self.gc();
            if cfg!(debug) {
                println!("GC, freed {} items", n);
            }
        }

        let obj = Rc::new(RefCell::new(Box::new(Object::new(t))));
        self.live_objects.push(obj.clone());
        obj
    }

    #[inline(always)]
    pub fn gc_disable(&mut self) {
        self.gc_disabled = true;
    }
    #[inline(always)]
    pub fn gc_enable(&mut self) {
        self.gc_disabled = false;
    }

    fn gc(&mut self) -> usize {
        if self.gc_disabled {
            return 0
        }
        let mut count = 0;
        self.environment.mark_all();
        let mut indices = Vec::<usize>::new();

        for i in 0..self.live_objects.len() {
            if !self.live_objects[i].borrow().marked {
                self.bytes_alloc -= self.live_objects[i].borrow().object_type.size_of();
                assert_eq!(Rc::strong_count(&self.live_objects[i]), 1);
                assert_eq!(Rc::weak_count(&self.live_objects[i]), 0);
                indices.push(i);
                count += 1;
                continue;
            }

            self.live_objects[i].borrow_mut().marked = false;
        }

        indices.reverse();
        for i in indices {
            self.live_objects.swap_remove(i);
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

        let mut iter = exp.iter();
        iter.next();
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

        if let Option::None = frontopt { //empty list
            return Result::Ok(self.new_nil());
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
        interpreter.environment.insert_sym("test".to_string(), obj);
        assert_eq!(interpreter.gc(), 0);
        assert_eq!(interpreter.live_objects.len(), 1);
        interpreter.environment.pop();
        assert_eq!(interpreter.gc(), 1);
        assert_eq!(interpreter.live_objects.len(), 0);

        for _ in 0..10 {
            interpreter.new_object(Type::String("foobar".to_string()));
        }

        assert_eq!(interpreter.gc(), 10);
        assert_eq!(interpreter.gc(), 0);
    }

    #[test]
    fn test_sym_found() {
        let mut interpreter = Interpreter::new();
        let obj = interpreter.new_object(Type::String("foobar".to_string()));
        interpreter.environment.insert_sym("test".to_string(), obj);
        interpreter.environment.find_sym("test".to_string()).expect("");
    }

    #[should_panic]
    #[test]
    fn test_sym_not_found() {
        let interpreter = Interpreter::new();
        interpreter.environment.find_sym("abcd".to_string()).expect("");
    }
}
