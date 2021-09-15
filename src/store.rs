use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use blocking::unblock;

/// Get the default store implementation for the current platform
///
/// Currently on non-WASM platforms this is the [`SimpleFsStore`], configured to use the `./data`
/// directory for storage.
///
/// The default store implementation on WASM will use the browser's IndexedDB, but this is not yet
/// implemented.
pub async fn get_default_store() -> Result<impl Store, StoreError> {
    #[cfg(not(target_arch = "wasm32"))]
    let store = SimpleFsStore::new(Path::new("./data")).await;

    #[cfg(target_arch = "wasm32")]
    let store = Ok(IndexedDbStore);

    store
}

#[cfg(target_arch = "wasm32")]
struct IndexedDbStore;

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait]
impl Store for IndexedDbStore {
    async fn get_string(&self, key: &str) -> Result<Option<String>, StoreError> {
        todo!()
    }

    async fn get_binary(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        todo!()
    }

    async fn put_string(&self, key: &str, data: &str) -> Result<(), StoreError> {
        todo!()
    }

    async fn put_binary(&self, key: &str, data: &[u8]) -> Result<(), StoreError> {
        todo!()
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        todo!()
    }
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
/// data store.
///
/// Calling [`Store::get_binary`] on a key on a string value or calling [`Store::get_string`] on a
/// binary value may either fail with an error, or simply interpert the contents of the key as the
/// other value type depending on the implementation. In-general you should only call the "get"
/// function that corresponds with the value at the given key to avoid unexpected behavior.
#[async_trait::async_trait]
pub trait Store {
    /// Get a string value from the store
    ///
    /// # Errors
    ///
    /// - This function **will** error if there is a problem accessing the store
    /// - This funciton **may** error or return an unexpected value if you try to read a key that
    ///   has a binary value currently set.
    async fn get_string(&self, key: &str) -> Result<Option<String>, StoreError>;
    /// Get a binary value from the store
    ///
    /// # Errors
    ///
    /// - This function **will** error if there is a problem accessing the store
    /// - This funciton **may** error or return an unexpected value if you try to read a key that
    ///   has a string value currently set.
    async fn get_binary(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError>;
    /// Store a string value in the store
    async fn put_string(&self, key: &str, data: &str) -> Result<(), StoreError>;
    /// Store a binary value in the store
    async fn put_binary(&self, key: &str, data: &[u8]) -> Result<(), StoreError>;
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
    /// Other, implementation-specific error
    #[error("Error: {0}")]
    Other(#[from] Box<dyn std::error::Error + Sync + Send>),
}

/// Ultra-simple filesystem [`Store`] implementation that uses a separate file for each string key
///
/// The names of each file will be the base64-encoded key and the value will be the string or binary
/// data associated to the key.
pub struct SimpleFsStore {
    /// The root directory for the filesystem store
    root_dir: PathBuf,
}

impl SimpleFsStore {
    /// Create a new [`FsStore`] that puts files in the specified `root_dir`
    pub async fn new(root_dir: &Path) -> Result<SimpleFsStore, StoreError> {
        let root_dir = root_dir.to_owned();

        unblock(move || {
            fs::create_dir_all(&root_dir).map_err(box_error)?;

            Ok(Self {
                root_dir: root_dir.to_owned(),
            })
        })
        .await
    }
}

#[async_trait::async_trait]
impl Store for SimpleFsStore {
    async fn get_binary(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        // Get the path to the file
        let file_path = self.root_dir.join(key);

        // Perform blocking operations on a thread pool
        unblock(move || {
            // Check if the file exists
            if !file_path.exists() {
                return Ok(None);
            }

            // Open the file
            let mut file = OpenOptions::new()
                .read(true)
                .open(file_path)
                .map_err(box_error)?;

            // Read the file into buffer
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map_err(box_error)?;

            // And return the buffer
            Ok(Some(buf))
        })
        .await
    }

    async fn put_binary(&self, key: &str, data: &[u8]) -> Result<(), StoreError> {
        // Get the path to the file
        let file_path = self.root_dir.join(key);
        // Clone the data
        let data = data.to_vec();

        // Perform blocking operations on a thread pool
        unblock(move || {
            // Open the file
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(file_path)
                .map_err(box_error)?;

            // Write the data to the file
            Ok(file.write_all(&data).map_err(box_error)?)
        })
        .await
    }

    async fn get_string(&self, key: &str) -> Result<Option<String>, StoreError> {
        // Get the binary data if it exists
        if let Some(binary) = self.get_binary(key).await? {
            // Parse the binary data as a UTF-8 String
            Ok(Some(String::from_utf8(binary).map_err(box_error)?))

        // The key doesn't exist
        } else {
            Ok(None)
        }
    }

    async fn put_string(&self, key: &str, data: &str) -> Result<(), StoreError> {
        // Store the string as UTF-8 bytes
        self.put_binary(key, data.as_bytes()).await
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        // Get the path to the file
        let file_path = self.root_dir.join(key);

        // Perform blocking operation on a thread pool
        unblock(move || {
            // Delete the file
            fs::remove_file(file_path).map_err(box_error)?;

            // Write the data to the file
            Ok(())
        })
        .await
    }
}

/// Helper to box the an error
fn box_error(
    e: impl std::error::Error + Sync + Send + 'static,
) -> Box<dyn std::error::Error + Sync + Send> {
    Box::new(e)
}
