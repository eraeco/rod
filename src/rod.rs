use crate::object::Object;
use crate::object::Value;
use crate::all::All;

pub struct Rod {
  all: All
}
impl Rod {
  pub fn new() -> Rod {
    let mut all = All::new();
    Rod {all: all}
  }
  pub fn get(&mut self, id: &str){
  }
  pub fn put(all: &mut All, id: &str, has: &str, value: Value){
    let mut obj = Object::new();
    obj.insert(id.to_string(), value);
    all.insert(has.to_string(), obj);
  }
}