//! Data structures used to build the rod data graph

use std::collections::HashMap;

use ulid::Ulid;

pub mod repr;

/// [`Node`] is the core data structure in the data graph
///
/// A graph is made up of a collection of nodes
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
    }

    //
    // Value
    //

    impl From<()> for Value {
        fn from(_: ()) -> Self {
            Self::None
        }
    }

    macro_rules! from_int {
        ($int:ident) => {
            impl From<$int> for Value {
                fn from(i: $int) -> Self {
                    Self::Int(i as i64)
                }
            }
        };
    }

    macro_rules! from_float {
        ($float:ident) => {
            impl From<$float> for Value {
                fn from(f: $float) -> Self {
                    Self::Float(f as f64)
                }
            }
        };
    }

    from_int!(i8);
    from_int!(i16);
    from_int!(i32);
    from_int!(i64);
    from_int!(u8);
    from_int!(u16);
    from_int!(u32);
    from_int!(u64);
    from_float!(f32);
    from_float!(f64);

    impl From<String> for Value {
        fn from(s: String) -> Self {
            Self::String(s)
        }
    }

    impl From<Vec<u8>> for Value {
        fn from(b: Vec<u8>) -> Self {
            Self::Binary(b)
        }
    }

    impl From<&Node> for Value {
        fn from(n: &Node) -> Self {
            Self::Node(n.id.clone())
        }
    }
}
