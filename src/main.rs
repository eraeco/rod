pub mod object;
pub mod all;
pub mod rod;

use crate::object::Value;
use crate::object::Object;
use crate::all::All;
use crate::rod::Rod;

fn main() {
  let mut all = All::new();
  Rod::put(&mut all, "todo", "milk", Value::Text("1gallon whole".to_string()));
  for id in all.keys() {
    println!("{}: {:?}", id, all.get(id));
  }
}