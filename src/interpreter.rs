use types::{Object, Type, HeapObject, Lambda, Procedure, List, new_list};
use error::{Err, ErrType};
use environment::Environment;
use std::option::Option;
use std::result::Result;
use std::rc::Rc;

pub struct Interpreter {
    live_objects: Vec<HeapObject>,
    fn_stack: Vec<Rc<String>>,
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
            fn_stack: Vec::new(),
            environment: Environment::new(),
            nil: Rc::new(Box::new(Object::new(Type::Cons(Box::new(new_list()))))),
            bool_true: Rc::new(Box::new(Object::new(Type::Bool(true)))),
            bool_false: Rc::new(Box::new(Object::new(Type::Bool(false)))),
            gc_disabled: false,
            bytes_alloc: 0,
            gc_threshold: 1000,
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
                debug_assert_eq!(Rc::strong_count(obj), 1);
                debug_assert_eq!(Rc::weak_count(obj), 0);
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
            return Result::Err(Err::new(
                ErrType::WrongArgsNum{wanted: params.len(), got: exp.len()-1},
                self.fn_stack.clone()));
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

        if let Type::Procedure(ref p) = last.as_ref().unwrap().clone().object_type {
            if let Procedure::Lambda(ref l) = *p.as_ref() {
                let closure = Lambda{
                        env: Option::Some(self.environment.cur_env_pop()),
                        params: l.params.clone(),
                        body: l.body.clone(),
                };
                last = Result::Ok(
                    self.new_object(Type::Procedure(Box::new(Procedure::Lambda(closure))))
                )
            }
        } else if let Option::Some(_) = lambda.env {
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

        let front = try!(self.eval(frontopt.unwrap().clone()));
        frontopt.map(|f| {
            if let Type::Symbol(ref s) = f.object_type {
                if let Type::Procedure(_) = front.object_type {
                    self.fn_stack.push(s.clone());
                }
            }
        });

        let res = match front.object_type {
            Type::Procedure(ref p) => match *p.as_ref() {
                Procedure::Primitive(prim) => prim(c),
                Procedure::Lambda(ref lambda) => self.eval_lambda(lambda, c.clone())
            },
            _ => Result::Err(Err::new(
                ErrType::NotCallable(front.get_type_string()),
                self.fn_stack.clone()))
        };

        let _ = res.as_ref().map(|_| {self.fn_stack.pop()});
        res
    }

    fn eval_body(&mut self, body: &List) -> Result<HeapObject, Err> {
        let mut last = Result::Ok(self.new_nil());
        for obj in body {
            last = Result::Ok(try!(self.eval(obj.clone())));
        }

        last
    }

    pub fn eval(&mut self, hobj: HeapObject) -> Result<HeapObject, Err> {
        match hobj.object_type {
            Type::Cons(ref c) => self.eval_cons(c),
            Type::Symbol(ref sym) => {
                let res = self.environment.find_sym(sym.clone());
                match res {
                    Result::Ok(val) => Result::Ok(val.clone()),
                    Result::Err(errt) => Result::Err(Err::new(errt, self.fn_stack.clone()))
                }
            },
            _ => Result::Ok(hobj.clone()),
        }
    }

    #[inline]
    fn check_args(&mut self, needed: usize, got: usize) -> Result<(), Err> {
        if needed != got {
            Result::Err(Err::new(ErrType::WrongArgsNum{wanted: needed, got: got}, self.fn_stack.clone()))
        } else {
            Result::Ok(())
        }
    }

    #[inline]
    fn check_min_args(&mut self, min: usize, got: usize) -> Result<(), Err> {
        if min > got {
            Result::Err(Err::new(ErrType::WrongMinArgsNum{min: 1, got: 0}, self.fn_stack.clone()))
        } else {
            Result::Ok(())
        }
    }

    #[inline]
    fn get_sym(&mut self, obj: HeapObject) -> Result<Rc<String>, Err> {
        if let Type::Symbol(ref s) = obj.as_ref().object_type {
            Result::Ok(s.clone())
        } else {
            Result::Err(Err::new(
                ErrType::WrongType{wanted: "symbolp",
                                   got: obj.clone().get_type_string()},
                                 self.fn_stack.clone()))
        }
    }

    //builtins
    pub fn print(&mut self, args: &List) -> Result<HeapObject, Err> {
        try!(self.check_min_args(1, args.len()));
        for obj in args {
            if let Type::Symbol(_) = obj.clone().object_type {
                print!("{} ", try!(self.eval(obj.clone())));
            }
            print!("{} ", try!(self.eval(obj.clone())));
        }
        Result::Ok(self.new_nil())
    }

    pub fn define(&mut self, args: &List) -> Result<HeapObject, Err> {
        try!(self.check_args(2, args.len()));
        let sym = try!(self.get_sym(args.front().unwrap().clone()));
        let val = try!(self.eval(args.iter().next().unwrap().clone()));
        self.environment.insert_sym(sym, val);
        Result::Ok(self.new_nil())
    }

    pub fn add(&mut self, args: &List) -> Result<HeapObject, Err> {
        let res = Object::add_list(args);
        match res {
            Result::Ok(obj) => Result::Ok(self.new_object(obj.object_type)),
            Result::Err(e) => Result::Err(Err::new(e, self.fn_stack.clone())),
        }
    }

    pub fn sub(&mut self, args: &List) -> Result<HeapObject, Err> {
        let res = Object::sub_list(args);
        match res {
            Result::Ok(obj) => Result::Ok(self.new_object(obj.object_type)),
            Result::Err(e) => Result::Err(Err::new(e, self.fn_stack.clone())),
        }
    }

    pub fn mul(&mut self, args: &List) -> Result<HeapObject, Err> {
        let res = Object::mul_list(args);
        match res {
            Result::Ok(obj) => Result::Ok(self.new_object(obj.object_type)),
            Result::Err(e) => Result::Err(Err::new(e, self.fn_stack.clone())),
        }
    }

    pub fn div(&mut self, args: &List) -> Result<HeapObject, Err> {
        let res = Object::div_list(args);
        match res {
            Result::Ok(obj) => Result::Ok(self.new_object(obj.object_type)),
            Result::Err(e) => Result::Err(Err::new(e, self.fn_stack.clone())),
        }
    }

    pub fn refcount(&mut self, args: &List) -> Result<HeapObject, Err> {
        try!(self.check_args(1, args.len()));
        let obj: &HeapObject = args.front().unwrap();
        Result::Ok(self.new_object(Type::Integer(Rc::strong_count(obj) as i64)))
    }

    pub fn eval_pub(&mut self, args: &List) -> Result<HeapObject, Err> {
        try!(self.check_args(1, args.len()));
        self.eval(args.front().unwrap().clone())
    }

    pub fn while_loop(&mut self, args: &List) -> Result<HeapObject, Err> {
        try!(self.check_args(2, args.len()));
        let mut last = Result::Ok(self.new_nil());
        let body = args.iter().next().unwrap();

        while try!(self.eval(args.front().unwrap().clone())).is_true() {
            if let Type::Cons(ref c) = body.clone().object_type {
                if let Type::Cons(_) = c.front().unwrap().object_type {
                    last = Result::Ok(try!(self.eval_body(body.unwrap_list())));
                }
            } else {
                last = Result::Ok(try!(self.eval(body.clone())));
            }
        }

        last
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use types::Type;
    use std::rc::Rc;
    use std::string::ToString;

    #[test]
    fn test_gc() {
        let mut interpreter = Interpreter::new();

        let obj = interpreter.new_object(Type::String(Rc::new("foobar".to_string())));
        interpreter.environment.push();
        interpreter.environment.insert_sym(Rc::new("test".to_string()), obj);
        assert_eq!(interpreter.gc(), 0);
        assert_eq!(interpreter.live_objects.len(), 1);
        interpreter.environment.pop();
        assert_eq!(interpreter.gc(), 1);
        assert_eq!(interpreter.live_objects.len(), 0);

        interpreter.gc_disable();
        for _ in 0..10 {
            interpreter.new_object(Type::String(Rc::new("foobar".to_string())));
        }
        interpreter.gc_enable();

        assert_eq!(interpreter.gc(), 10);
        assert_eq!(interpreter.gc(), 0);
    }

    #[test]
    fn test_sym_found() {
        let mut interpreter = Interpreter::new();
        let obj = interpreter.new_object(Type::String(Rc::new("foobar".to_string())));
        interpreter.environment.insert_sym(Rc::new("test".to_string()), obj);
        interpreter.environment.find_sym(Rc::new("test".to_string())).expect("");
    }

    #[should_panic]
    #[test]
    fn test_sym_not_found() {
        let interpreter = Interpreter::new();
        interpreter.environment.find_sym(Rc::new("abcd".to_string())).expect("");
    }
}
