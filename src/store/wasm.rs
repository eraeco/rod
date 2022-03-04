//! Store implementatons for WASM targets

use super::{Store, StoreError};

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

    async fn put_radix(
        &self,
        _tree: radix_trie::Trie<String, ulid::Ulid>,
    ) -> Result<(), StoreError> {
        todo!()
    }

    async fn get_radix(&self) -> Result<radix_trie::Trie<String, ulid::Ulid>, StoreError> {
        todo!()
    }
}
