use types::{Object, alloc_object};
use std::collections::HashMap;
use std::option::Option;

pub struct Interpreter {
    live_objects: Box<Vec<alloc_object>>,
    environment: Box<Vec<HashMap<String, alloc_object>>>,
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

    pub fn new_object(&mut self, object: alloc_object) {
        let obj = object;
        self.live_objects.push(obj);
    }

    pub fn env_push(&mut self) {
        let hash = HashMap::<String, alloc_object>::new();
        self.environment.push(hash);
    }

    pub fn env_pop(&mut self) {
        match self.environment.pop() {
            Some(val) => val,
            None => panic!("root environment removed, this shouldn't happen"),
        };
    }

    pub fn insert_symbol(&mut self, name: String, value: alloc_object) {
        let index = self.environment.len()-1;
        self.environment[index].insert(name, value);
    }

    // insert symbol in last to last env
    pub fn insert_symbol_prev(&mut self, name: String, value: alloc_object) {
        let index = self.environment.len()-2;
        self.environment[index].insert(name, value);
    }

    pub fn find_symbol(&self, name: String) -> Option<&alloc_object> {
        let index = self.environment.len()-1;

        self.environment[index].get(&name)
    }

    pub fn find_symbol_prev(&self, name: String) -> Option<&alloc_object> {
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
    use types::{Object, Type};
    use std::string::ToString;
    
    #[test]
    fn test_gc() {
        let mut obj = Object::new(Type::String("abcd".to_string()));
        let mut interpreter = Interpreter::new();
        interpreter.new_object(obj.clone());
        interpreter.insert_symbol("test".to_string(), obj.clone());
        assert_eq!(interpreter.gc(), 0);
        
        obj= Object::new(Type::String("abcd".to_string()));
        interpreter.new_object(obj.clone());
        assert_eq!(interpreter.gc(), 1);
    }
}
