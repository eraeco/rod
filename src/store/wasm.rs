//! Store implementatons for WASM targets

use super::{Store, StoreError};

use crate::graph::Value;

/// Get the default WASM store
pub async fn get_default_store() -> Result<impl Store, StoreError> {
    Ok(IndexedDbStore)
}

pub struct IndexedDbStore;

#[async_trait::async_trait]
impl Store for IndexedDbStore {
    async fn get(&self, _key: &str) -> Result<Option<crate::graph::Node>, StoreError> {
        todo!()
    }

    async fn put(&self, _key: &str, _value: crate::graph::Node) -> Result<(), StoreError> {
        todo!()
    }

    async fn delete(&self, _key: &str) -> Result<(), StoreError> {
        todo!()
    }
}
