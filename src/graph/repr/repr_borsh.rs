//! [borsh] representation
//!
//! [borsh]: https://github.com/near/borsh

use borsh::{BorshDeserialize, BorshSerialize};

use std::collections::HashMap;

use super::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct BorshNode {
    pub id: u128,
    pub fields: HashMap<String, BorshField>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct BorshField {
    pub updated_at: f64,
    pub value: BorshValue,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum BorshValue {
    None,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Binary(Vec<u8>),
    Node(u128),
}

mod to_borsh {
    use super::*;

    impl From<Node> for BorshNode {
        fn from(node: Node) -> Self {
            Self {
                id: node.id.into(),
                fields: node
                    .fields
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect(),
            }
        }
    }
    impl From<Field> for BorshField {
        fn from(field: Field) -> Self {
            Self {
                updated_at: field.updated_at,
                value: field.value.into(),
            }
        }
    }
    impl From<Value> for BorshValue {
        fn from(value: Value) -> Self {
            match value {
                Value::None => BorshValue::None,
                Value::Bool(b) => BorshValue::Bool(b),
                Value::Int(i) => BorshValue::Int(i),
                Value::Float(f) => BorshValue::Float(f),
                Value::String(s) => BorshValue::String(s),
                Value::Binary(b) => BorshValue::Binary(b),
                Value::Node(n) => BorshValue::Node(n.into()),
            }
        }
    }
}

mod from_borsh {
    use super::*;

    impl From<BorshNode> for Node {
        fn from(node: BorshNode) -> Self {
            Self {
                id: node.id.into(),
                fields: node
                    .fields
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect(),
            }
        }
    }
    impl From<BorshField> for Field {
        fn from(field: BorshField) -> Self {
            Self {
                updated_at: field.updated_at,
                value: field.value.into(),
            }
        }
    }
    impl From<BorshValue> for Value {
        fn from(value: BorshValue) -> Self {
            match value {
                BorshValue::None => Value::None,
                BorshValue::Bool(b) => Value::Bool(b),
                BorshValue::Int(i) => Value::Int(i),
                BorshValue::Float(f) => Value::Float(f),
                BorshValue::String(s) => Value::String(s),
                BorshValue::Binary(b) => Value::Binary(b),
                BorshValue::Node(n) => Value::Node(n.into()),
            }
        }
    }
}
