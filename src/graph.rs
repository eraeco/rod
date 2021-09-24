//! Data structures used to build the rod data graph

use std::{collections::HashMap, ops::Deref};

use ulid::Ulid;

pub mod repr;

/// [`Node`] is the core data structure in the data graph
///
/// A grpah is made up of a collection of nodes
#[derive(Debug, Clone)]
pub struct Node {
    /// The node's universally unique identifier
    pub id: Ulid,
    /// The fields in the node
    pub fields: HashMap<String, Field>,
}

/// A [`Field`] is a named item in a node
///
/// A field encompases the last time that the field was modified, and the value of the field
#[derive(Debug, Clone)]
pub struct Field {
    /// The time in seconds that this field value was updated as relative to the
    /// [`UNIX_EPOCH`][std::time::SystemTime::UNIX_EPOCH]
    pub updated_at: f64,
    /// The value of the field
    pub value: Value,
}

/// A value represents the different data types that a field value can take
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// An empty value
    None,
    /// A boolean value
    Bool(bool),
    /// A signed integer value
    Int(i64),
    /// A floating point value
    Float(f64),
    /// A string value
    String(String),
    /// A binary data value
    Binary(Vec<u8>),
    /// A reference to the unique ID of another node
    Node(Ulid),
}

mod impls {
    use crate::{engine::Rod, store::StoreError};

    use super::*;

    //
    // Node
    //

    impl Default for Node {
        fn default() -> Self {
            Self {
                id: Ulid::new(),
                fields: Default::default(),
            }
        }
    }

    impl Node {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn new_with_fields(fields: Vec<(String, Field)>) -> Self {
            Self {
                id: Ulid::new(),
                fields: fields.into_iter().collect(),
            }
        }

        pub fn get(&self, key: &str) -> Option<&Value> {
            self.fields.get(key).map(|x| &x.value)
        }

        pub fn set<V: Into<Value>>(&mut self, key: &str, value: V) {
            self.fields.insert(key.into(), Field::new(value.into()));
        }
    }

    //
    // Field
    //

    impl Deref for Field {
        type Target = Value;

        fn deref(&self) -> &Self::Target {
            &self.value
        }
    }

    //
    // Value
    //

    impl Value {
        /// Return whether or not the value is [`Value::None`]
        pub fn is_none(&self) -> bool {
            self == &Value::None
        }

        /// Get the value as an ID
        pub fn as_id(&self) -> Option<&Ulid> {
            if let Value::Node(id) = self {
                Some(id)
            } else {
                None
            }
        }

        /// If this value is an ID, get the node associated to the ID
        pub async fn follow(&self, rod: &Rod) -> Result<Option<Node>, StoreError> {
            let id = if let Value::Node(id) = self {
                id
            } else {
                return Ok(None);
            };

            rod.get(id).await
        }
    }

    impl From<String> for Value {
        fn from(s: String) -> Self {
            Self::String(s)
        }
    }

    impl From<&Node> for Value {
        fn from(n: &Node) -> Self {
            Self::Node(n.id.clone())
        }
    }
}
