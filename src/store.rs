use crate::graph::Node;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod wasm;

/// Get the default store implementation for the current platform
///
/// Currently on non-WASM platforms this is the [`SimpleFsStore`], configured to use the `./data`
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

/// A simple, raw data storage interface
///
/// [`Store`] is designed to be implemented over any persistant storage interface such as the
/// filesystem, S3, browser LocalStorage or IndexedDB, etc.
///
/// The interface is a simple key-value store, where each key could either have no data asociated to
/// it, or it could have a value that may either be either string data or binary data.
///
/// Because some data stores such as browser local storage can only store string data, separate
/// functions are used for getting and setting string and binary data. This allows the
/// implementation to choose to base64 encode/decode binary values when writing to a string-only
/// data store, or to just write the raw bytes directly if binary storage is supported.
///
/// Calling [`Store::get_binary`] on a key on a string value or calling [`Store::get_string`] on a
/// binary value should fail with an error.
#[async_trait::async_trait]
pub trait Store {
    /// Get a value from the store
    async fn get(&self, key: &str) -> Result<Option<Node>, StoreError>;

    // Put a value in the store
    async fn put(&self, key: &str, value: Node) -> Result<(), StoreError>;

    /// Delete a key
    async fn delete(&self, key: &str) -> Result<(), StoreError>;
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
