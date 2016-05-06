use types::Type;

pub enum Err {
    WrongType{wanted: &'static str, got: &'static str}
}
