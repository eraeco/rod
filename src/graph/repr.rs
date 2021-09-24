//! Representations of the graph data structures used for serialization/deserialization

use super::*;
#[cfg(feature = "borsh")]
pub mod repr_borsh;

#[cfg(feature = "json")]
pub mod repr_json;
