//! Store implementatons for WASM targets

use super::Store;

/// Get the default WASM store
pub async fn get_default_store() -> Result<impl Store, StoreError> {
    let store = Ok(IndexedDbStore);
}

pub struct IndexedDbStore;

#[async_trait::async_trait]
impl Store for IndexedDbStore {
    async fn get(&self, key: &str) -> Result<Option<Value>, StoreError> {}
    async fn put(&self, key: &str, value: Value) -> Result<(), StoreError> {}
    async fn delete(&self, key: &str) -> Result<(), StoreError> {}
}
