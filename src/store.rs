//! Backing data stores used for persistant data

use ulid::Ulid;

use crate::graph::Node;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

/// Get the default store implementation for the current platform
///
/// Currently on non-WASM platforms this is the `SimpleFsStore`, configured to use the `./data`
/// directory for storage.
///
/// The default store implementation on WASM will use the browser's IndexedDB, but this is not yet
/// implemented.
pub async fn get_default_store() -> Result<impl Store, StoreError> {
    #[cfg(not(target_arch = "wasm32"))]
    let store = native::get_default_store().await;

    #[cfg(target_arch = "wasm32")]
    let store = wasm::get_default_store().await;

    store
}

/// A simple, [`Node`] storage interface
///
/// [`Store`] is designed to be implemented over any persistant storage interface such as the
/// filesystem, S3, browser LocalStorage or IndexedDB, etc.
#[async_trait::async_trait]
pub trait Store {
    /// Get a node from the store using it's ULID
    async fn get_node(&self, id: &Ulid) -> Result<Option<Node>, StoreError>;

    /// Put a node into the store
    ///
    /// The node can later be retrieved using it's ULID
    async fn put_node(&self, node: Node) -> Result<(), StoreError>;

    /// Delete a node from the store using it's ULID
    async fn delete_node(&self, id: &Ulid) -> Result<(), StoreError>;

    /// Point a string key in the database to a node's ULID
    async fn set_id(&self, key: &str, id: Option<Ulid>) -> Result<(), StoreError>;

    /// Get the ULID pointed at by the string key in the database
    async fn get_id(&self, key: &str) -> Result<Option<Option<Ulid>>, StoreError>;
}

/// An error that can occur in a [`Store`]
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Attempted to read a binary value as a string
    #[error("Attempted to read binary data as a string")]
    ReadBinaryAsString,
    /// Attempted to read a string value as binary
    #[error("Attempted to read string data as binary")]
    ReadStringAsBinary,
    /// Found unrecognized data in storage medium
    #[error("Found unrecognized data in storage medium")]
    UnrecognizedData,
    /// Other, implementation-specific error
    #[error("Error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Sync + Send>),
}
