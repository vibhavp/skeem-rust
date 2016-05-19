use types::{Object, Type, HeapObject, Lambda, Procedure, List, new_list};
use error::Err;
use environment::Environment;
use std::option::Option;
use std::result::Result;
use std::rc::Rc;

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
            nil: Rc::new(Box::new(Object::new(Type::Cons(Box::new(new_list()))))),
            bool_true: Rc::new(Box::new(Object::new(Type::Bool(true)))),
            bool_false: Rc::new(Box::new(Object::new(Type::Bool(false)))),
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

        let obj = Rc::new(Box::new(Object::new(t)));
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
            let ref obj = self.live_objects[i];
            if !obj.marked.get() {
                self.bytes_alloc -= obj.object_type.size_of();
                assert_eq!(Rc::strong_count(obj), 1);
                assert_eq!(Rc::weak_count(obj), 0);
                indices.push(i);
                count += 1;

            } else {
                obj.marked.set(false);
            }
        }

        indices.reverse();
        for i in indices {
            self.live_objects.swap_remove(i);
        }

        count
    }

    fn eval_lambda(&mut self, lambda: &Lambda, exp: List) -> Result<HeapObject, Err> {
        let params = lambda.params.unwrap_list().clone();
        if params.len() != exp.len() - 1 {
            return Result::Err(Err::WrongArgsNum{wanted: params.len(), got: exp.len()-1});
        }

        self.environment.push();
        if let Option::Some(ref env) = lambda.env {
            self.environment.push();
            for (sym, obj) in env.iter() {
                self.environment.insert_sym(sym.clone(), obj.clone());
            }
        }

        //let mut last = self.nil.clone();
        let mut last = Result::Ok(self.nil.clone());

        /* (lambda-obj p a r a m s)
         *              ^---------^
         *              params_iter()
         */
        let mut param_syms_iter = exp.iter();
        for supplied_param in params.iter() {
            let param_sym = param_syms_iter.next().unwrap();
            self.environment.insert_sym(param_sym.unwrap_sym(), supplied_param.clone());
        }

        // (lambda (a r g s) body)
        for obj in lambda.body.unwrap_list().iter() {
            last = self.eval(obj.clone());
            if let Result::Err(_) = last {
                break
            }
        }

        if let Option::Some(_) = lambda.env {
            self.environment.pop();
        }
        self.environment.pop();

        last
    }

    fn eval_cons(&mut self, c: &List) -> Result<HeapObject, Err> {
        let frontopt = c.front();

        if let Option::None = frontopt { //empty list
            return Result::Ok(self.new_nil());
        }

        let front = self.eval(frontopt.unwrap().clone());

        match front {
            Result::Ok(frontval) => {
                match frontval.object_type {
                    Type::Procedure(ref p) => match p.as_ref() {
                        &Procedure::Primitive(prim) => prim(c),
                        &Procedure::Lambda(ref lambda) => self.eval_lambda(lambda, c.clone())
                    },
                    _ => Result::Err(Err::NotCallable(frontval.get_type_string()))
                }
            },
            Result::Err(e) => Result::Err(e)
        }
    }

    pub fn eval(&mut self, hobj: HeapObject) -> Result<HeapObject, Err> {
        match hobj.object_type {
            Type::Cons(ref c) => self.eval_cons(c),
            Type::Symbol(ref sym) => Result::Ok(try!(self.environment.find_sym(sym.clone())).clone()),
            _ => Result::Ok(hobj.clone()),
        }
    }

    //builtins
    pub fn print() {

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
