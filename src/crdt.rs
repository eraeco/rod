//! CRDTs ( Conflict-free Replicated Data Types ), including the implementation of GUN's HAM
//! algorithm

use std::{cmp, mem, time::SystemTime};

use crate::graph::{Field, Value};

/// Trait implemented by structs that can be lexically sorted
pub trait LexicalCmp {
    /// Compare two object lexographically
    fn lexical_cmp(&self, other: &Self) -> cmp::Ordering;
}

/// If an update comes in that is more than this amount of time in the future, we will assume that
/// the node that sent the update is lying and trying to make it's update take precedence over the
/// current value of the field.
const FUTURE_UPDATE_THREASHOLD: f64 = 600.0;

impl Field {
    pub fn new(value: Value) -> Self {
        Self {
            updated_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Could not get system time")
                .as_secs_f64(),
            value,
        }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn state(&self) -> &f64 {
        &self.updated_at
    }

    /// Merge the new value into this field, using the [HAM] merge conflict resolution strategy
    ///
    /// [HAM]: https://github.com/amark/gun/wiki/Conflict-Resolution-with-Guns
    pub fn merge_with(&mut self, field: &Field) {
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("TODO: system time")
            .as_secs_f64();

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
