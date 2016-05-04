use std::collections::{LinkedList, HashMap};
use std::boxed::Box;
use std::rc::Rc;
use std::cell::RefCell;

pub type alloc_object = Rc<RefCell<Box<Object>>>;
pub type list = LinkedList<Object>;
pub type environment = HashMap<String, alloc_object>;

pub enum Type {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Character(char),
    String(String),

    Cons(Box<list>),
    Environment(environment),
    Procedure(Box<Proc>),
}

pub struct Object {
    object_type: Box<Type>,
    pub marked: bool,
}

pub struct Proc {
    params: Box<Vec<String>>,
    body: Box<list>
}

pub enum Procedure {
    Lambda {procedure: Proc},
    Closure {env: HashMap<String, Object>, procedure: Proc},
    Primitive(&'static Fn(list) -> Object)
}

impl Object {
    pub fn new(t: Type) -> Rc<RefCell<Box<Object>>> {
        Rc::new(RefCell::new(Box::new(Object{object_type: Box::new(t), marked: false})))
    }

    fn mark(&mut self) {
        if self.marked {
            return
        }

        self.marked = true;
        match self.object_type.as_mut() {
            &mut Type::Cons(ref mut cons) => Object::mark_list(cons),
            &mut Type::Environment(ref mut env) => Object::mark_environment(env),
            _ => {},
        };
    }

    pub fn mark_environment(env: &mut environment) {
        for (_, object) in env.iter_mut() {
            object.borrow_mut().marked = true;
        }
    }

    fn mark_list(cons: &mut list) {
        for obj in cons {
            obj.mark();
        }
    }
}
