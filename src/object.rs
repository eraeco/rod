use std::collections::HashMap;

pub type Object = HashMap<String, Value>;

#[derive(Debug)]
pub enum Value {
    Null(Option<bool>),
    Bit(bool),
    Number(f32),
    Text(String),
    Link(Object)
}