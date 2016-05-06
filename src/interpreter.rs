use types::{Object, Type, HeapObject, list};
use err::Err;

use std::collections::HashMap;
use std::option::Option;
use std::result::Result;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::LinkedList;

pub struct Interpreter {
    live_objects: Box<Vec<HeapObject>>,
    environment: Box<Vec<HashMap<String, HeapObject>>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut i = Interpreter{
            live_objects: Box::new(Vec::new()),
            environment: Box::new(Vec::with_capacity(1)),
        };

        i.env_push();
        return i;
    }

    pub fn new_object(&mut self, t: Type) -> HeapObject {
        let obj = Rc::new(RefCell::new(Box::new(Object::new(t))));
        self.live_objects.push(obj.clone());
        obj
    }

    fn env_push(&mut self) {
        let hash = HashMap::<String, HeapObject>::new();
        self.environment.push(hash);
    }

    fn env_pop(&mut self) {
        match self.environment.pop() {
            Some(val) => val,
            None => panic!("root environment removed, this shouldn't happen"),
        };
    }

    fn insert_symbol(&mut self, name: String, value: HeapObject) {
        let index = self.environment.len()-1;
        self.environment[index].insert(name, value);
    }

    // insert symbol in last to last env
    fn insert_symbol_prev(&mut self, name: String, value: HeapObject) {
        let index = self.environment.len()-2;
        self.environment[index].insert(name, value);
    }

    fn find_symbol(&self, name: String) -> Option<&HeapObject> {
        let index = self.environment.len()-1;

        self.environment[index].get(&name)
    }

    fn find_symbol_prev(&self, name: String) -> Option<&HeapObject> {
        let index = self.environment.len()-2;

        self.environment[index].get(&name)
    }

    fn mark_all(&mut self) {
        for env in self.environment.iter_mut() {
            Object::mark_environment(env);
        }
    }

    fn gc(&mut self) -> u64 {
        let mut count = 0 as u64;
        self.mark_all();

        for i in 0..self.live_objects.len() {
            if !self.live_objects[i].borrow_mut().marked {
                self.live_objects.swap_remove(i);
                count += 1;
            }
        }
        count
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use types::Type;
    use std::string::ToString;
    use std::rc::Rc;

    #[test]
    fn test_gc() {
        let mut interpreter = Interpreter::new();

        {
            {
                let mut obj = interpreter.new_object(Type::String("foobar".to_string()));
                interpreter.insert_symbol("test".to_string(), obj.clone());
            }
            assert_eq!(interpreter.gc(), 0);

            let mut obj = interpreter.new_object(Type::String("foobar".to_string()));
            assert_eq!(interpreter.gc(), 1);
            assert_eq!(interpreter.gc(), 0);
        }

        for obj in interpreter.live_objects.into_iter() {
            assert_eq!(Rc::strong_count(&obj), 2);
            assert_eq!(Rc::weak_count(&obj), 0);
        }
    }
}
