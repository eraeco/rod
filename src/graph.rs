//! Data structures used to build the rod data graph

use std::{
    cmp,
    collections::HashMap,
    mem,
    ops::Deref,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait LexicalCmp {
    fn lexical_cmp(&self, other: &Self) -> cmp::Ordering;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub fields: HashMap<String, Field>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            fields: Default::default(),
        }
    }
}

impl Node {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Time that this field value was updated as relative to the
    /// [`UNIX_EPOCH`][std::time::SystemTime::UNIX_EPOCH]
    updated_at: Duration,
    value: Value,
}

/// If an update comes in that is more than this amount of time in the future, we will assume that
/// the node that sent the update is lying and trying to make it's update take precedence over the
/// current value of the field.
const FUTURE_UPDATE_THREASHOLD: Duration = Duration::from_secs(60 * 10);

impl Field {
    /// Merge the new value into this field, using the [HAM] merge conflict resolution strategy
    ///
    /// [HAM]: https://github.com/amark/gun/wiki/Conflict-Resolution-with-Guns
    pub fn merge_with(&mut self, field: &Field) {
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("TODO: system time");

        // If the new field has the same timestamp
        if field.updated_at == self.updated_at {
            match self.lexical_cmp(field) {
                // Totally equal, do nothing
                cmp::Ordering::Equal => return,
                // Keep our value, do nothing
                cmp::Ordering::Less => return,
                // Keep the new value
                cmp::Ordering::Greater => self.value = field.value.clone(),
            }

        // If the other field is and older update than the one we have, just ignore it
        } else if field.updated_at < self.updated_at {
            return;

        // If the other field is in the future
        } else if field.updated_at > current_time {
            // If the field is too far in the future, ignore it
            if field.updated_at - current_time > FUTURE_UPDATE_THREASHOLD {
                return;
            }

            // Wait to apply this update until later
            unimplemented!(
                "Use async logic to apply this update once our system clock \
                reaches the future time"
            );
        } else {
            unreachable!()
        }
    }
}

impl Field {
    pub fn new(value: Value) -> Self {
        Self {
            updated_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Could not get system time"),
            value,
        }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn state(&self) -> &Duration {
        &self.updated_at
    }
}

impl Deref for Field {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Binary(Vec<u8>),
    Node(Uuid),
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Self::Int(i as i64)
    }
}

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Self::Float(f as f64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<Vec<u8>> for Value {
    fn from(b: Vec<u8>) -> Self {
        Self::Binary(b)
    }
}

impl From<Uuid> for Value {
    fn from(u: Uuid) -> Self {
        Self::Node(u)
    }
}

impl LexicalCmp for Value {
    fn lexical_cmp(&self, other: &Self) -> cmp::Ordering {
        use Value::*;
        if mem::discriminant(self) == mem::discriminant(other) {
            match (self, other) {
                (Bool(x), Bool(y)) => match (x, y) {
                    (true, true) => cmp::Ordering::Equal,
                    (true, false) => cmp::Ordering::Less,
                    (false, true) => cmp::Ordering::Greater,
                    (false, false) => cmp::Ordering::Equal,
                },
                (Int(x), Int(y)) => x.cmp(y),
                (Float(x), Float(y)) => x.partial_cmp(y).unwrap_or(cmp::Ordering::Less),
                (String(x), String(y)) => x.cmp(y),
                (Binary(x), Binary(y)) => x.cmp(y),
                (Node(x), Node(y)) => x.cmp(y),
                _ => unreachable!(),
            }
        } else {
            let self_rank = match self {
                None => 0,
                Bool(_) => 1,
                Int(_) => 2,
                Float(_) => 3,
                String(_) => 4,
                Binary(_) => 5,
                Node(_) => 6,
            };
            let other_rank = match other {
                None => 0,
                Bool(_) => 1,
                Int(_) => 2,
                Float(_) => 3,
                String(_) => 4,
                Binary(_) => 5,
                Node(_) => 6,
            };

            if self_rank < other_rank {
                cmp::Ordering::Less
            } else if self_rank > other_rank {
                cmp::Ordering::Greater
            } else {
                unreachable!()
            }
        }
    }
}
