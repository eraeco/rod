//! Representations of the graph data structures used for serialization/deserialization

use super::*;

/// [borsh] representation
///
/// [borsh]: https://github.com/near/borsh
#[cfg(feature = "borsh")]
pub mod repr_borsh {
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
}

#[cfg(feature = "json")]
pub mod repr_json {
    use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

    use std::{collections::HashMap, convert::TryFrom};

    use super::*;

    #[derive(Deserialize, Serialize)]
    pub struct JsonNode {
        #[serde(rename = "_")]
        pub meta: JsonNodeMeta,
        #[serde(flatten)]
        pub fields: HashMap<String, JsonValue>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct JsonNodeMeta {
        #[serde(rename = "#")]
        id: Ulid,
        #[serde(rename = ">")]
        field_states: HashMap<String, f64>,
    }

    pub enum JsonValue {
        None,
        Bool(bool),
        Int(i64),
        Float(f64),
        String(String),
        Binary { data: BinaryData },
        Node { id: Ulid },
    }

    #[derive(Serialize, Deserialize, Clone)]
    #[serde(try_from = "String", into = "String")]
    pub struct BinaryData(Vec<u8>);

    impl Into<String> for BinaryData {
        fn into(self) -> String {
            base64::encode(self.0)
        }
    }

    impl TryFrom<String> for BinaryData {
        type Error = &'static str;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            base64::decode(value)
                .map(|x| BinaryData(x))
                .map_err(|_| "String is not valid base64")
        }
    }

    mod to_json {
        use super::*;

        impl From<Node> for JsonNode {
            fn from(node: Node) -> Self {
                let mut field_states = HashMap::with_capacity(node.fields.len());
                let mut fields = HashMap::with_capacity(node.fields.len());
                for (k, v) in node.fields {
                    field_states.insert(k.clone(), v.updated_at);
                    fields.insert(k, v.value.into());
                }
                Self {
                    meta: JsonNodeMeta {
                        id: node.id,
                        field_states,
                    },
                    fields,
                }
            }
        }

        impl From<Value> for JsonValue {
            fn from(value: Value) -> Self {
                match value {
                    Value::None => JsonValue::None,
                    Value::Bool(b) => JsonValue::Bool(b),
                    Value::Int(i) => JsonValue::Int(i),
                    Value::Float(f) => JsonValue::Float(f),
                    Value::String(s) => JsonValue::String(s),
                    Value::Binary(b) => JsonValue::Binary {
                        data: BinaryData(b),
                    },
                    Value::Node(n) => JsonValue::Node { id: n.into() },
                }
            }
        }
    }

    mod from_json {
        use super::*;

        impl From<JsonNode> for Node {
            fn from(node: JsonNode) -> Self {
                let JsonNode { mut meta, fields } = node;
                Self {
                    id: meta.id.into(),
                    fields: fields
                        .into_iter()
                        .map(|(k, v)| {
                            let field = Field {
                                updated_at: meta.field_states.remove(&k).unwrap(),
                                value: v.into(),
                            };
                            (k, field)
                        })
                        .collect(),
                }
            }
        }
        impl From<JsonValue> for Value {
            fn from(value: JsonValue) -> Self {
                match value {
                    JsonValue::None => Value::None,
                    JsonValue::Bool(b) => Value::Bool(b),
                    JsonValue::Int(i) => Value::Int(i),
                    JsonValue::Float(f) => Value::Float(f),
                    JsonValue::String(s) => Value::String(s),
                    JsonValue::Binary { data } => Value::Binary(data.0),
                    JsonValue::Node { id } => Value::Node(id.into()),
                }
            }
        }
    }

    mod serde_impls {
        use serde::ser::SerializeMap;

        use super::*;

        impl Serialize for JsonValue {
            fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    JsonValue::None => s.serialize_none(),
                    JsonValue::Bool(b) => s.serialize_bool(*b),
                    JsonValue::Int(i) => s.serialize_i64(*i),
                    JsonValue::Float(f) => s.serialize_f64(*f),
                    JsonValue::String(string) => s.serialize_str(string),
                    JsonValue::Binary { data } => {
                        s.serialize_str(&format!("$base64${}", base64::encode(data.0.clone())))
                    }
                    JsonValue::Node { id } => {
                        let mut map = s.serialize_map(Some(1))?;
                        map.serialize_entry("#", &id.to_string())?;
                        map.end()
                    }
                }
            }
        }

        struct JsonValueVisitor;

        macro_rules! visit_int {
            ($fn:ident, $int:ident) => {
                fn $fn<E>(self, v: $int) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(JsonValue::Int(v as i64))
                }
            };
        }
        macro_rules! visit_float {
            ($fn:ident, $float:ident) => {
                fn $fn<E>(self, v: $float) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(JsonValue::Float(v as f64))
                }
            };
        }

        impl<'de> Visitor<'de> for JsonValueVisitor {
            type Value = JsonValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                "null, a boolean, an integer, a float, a string, base64 encoded binary data as a \
                string starting with `$base64$`, or a map with a single field `#` set to a uuid",
            )
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(JsonValue::Bool(v))
            }

            visit_int!(visit_i64, i64);
            visit_int!(visit_i32, i32);
            visit_int!(visit_i16, i16);
            visit_int!(visit_i8, i8);
            visit_int!(visit_u64, u64);
            visit_int!(visit_u32, u32);
            visit_int!(visit_u16, u16);
            visit_int!(visit_u8, u8);
            visit_float!(visit_f32, f32);
            visit_float!(visit_f64, f64);

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let base64_prefix = "$base64$";
                if v.starts_with(base64_prefix) {
                    let data_base64 = v.strip_prefix(base64_prefix).unwrap_or("");

                    let data = base64::decode(data_base64).map_err(|_| {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Str("$base64$[invalid base64 data]"),
                            &"valid base64 encoded data",
                        )
                    })?;

                    Ok(JsonValue::Binary {
                        data: BinaryData(data),
                    })
                } else {
                    Ok(JsonValue::String(v.to_owned()))
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&v)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(JsonValue::None)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(JsonValue::None)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                if let Some(key) = map.next_key()? {
                    if key == String::from("#") {
                        let ulid_str: &str = map.next_value()?;
                        Ok(JsonValue::Node {
                            id: Ulid::from_string(ulid_str).map_err(|_| {
                                serde::de::Error::invalid_value(
                                    serde::de::Unexpected::Str(ulid_str),
                                    &"Valid ULID",
                                )
                            })?,
                        })
                    } else {
                        Err(serde::de::Error::unknown_field(key, &["#"]))
                    }
                } else {
                    Err(serde::de::Error::missing_field("#"))
                }
            }
        }

        impl<'de> Deserialize<'de> for JsonValue {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                d.deserialize_any(JsonValueVisitor)
            }
        }
    }
}
