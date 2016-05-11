use std::collections::HashMap;
use std::result::Result;
use std::option::Option;
use types::HeapObject;
use error::Err;

pub struct Environment(Vec<HashMap<String, HeapObject>>);

impl Environment {
    pub fn new() -> Environment {
        let mut e = Environment(Vec::with_capacity(1));
        e.push();
        e
    }

    #[inline(always)]
    pub fn push(&mut self) {
        self.0.push(HashMap::<String, HeapObject>::new());
    }

    #[inline(always)]
    pub fn pop(&mut self) {
        self.0.pop().expect("popping the root environment");
    }

    #[inline(always)]
    pub fn insert_sym(&mut self, name: String, value: HeapObject) {
        self.0.last_mut().unwrap().insert(name, value);
    }

    pub fn find_sym(&self, name: String) -> Result<&HeapObject, Err> {
        if self.0.len() == 1 {
            let val = self.0[0].get(&name);
            return match val {
                Option::Some(val) => Result::Ok(val),
                Option::None => Result::Err(Err::SymbolNotFound(name))
            }
        }

        for i in self.0.len()-1..0 {
            if let Option::Some(val) = self.0[i].get(&name) {
                return Result::Ok(val)
            }
        }

        Result::Err(Err::SymbolNotFound(name))
    }

    pub fn mark_all(&mut self) {
        for env in self.0.iter_mut() {
            for (_, object) in env {
                object.borrow_mut().mark();
            }
        }
    }
}
