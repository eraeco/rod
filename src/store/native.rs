//! Store implementatons for native targets

use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use blocking::unblock;

use crate::graph::{repr::repr_borsh::BorshNode, Node};

use super::{Store, StoreError};

/// Get the default native store
pub async fn get_default_store() -> Result<impl Store, StoreError> {
    SimpleFsStore::new(Path::new("./data")).await
}

// TODO: SledDB filesystem store

/// Ultra-simple filesystem [`Store`] implementation that uses a separate file for each string key
///
/// The names of each file will be the base64-encoded key and the value will be the string or binary
/// data associated to the key.
pub struct SimpleFsStore {
    /// The root directory for the filesystem store
    root_dir: PathBuf,
}

impl SimpleFsStore {
    /// Create a new [`SimpleFsStore`] that puts files in the specified `root_dir`
    pub async fn new(root_dir: &Path) -> Result<SimpleFsStore, StoreError> {
        #[cfg(not(feature = "borsh"))]
        compile_error!("`borsh` feature required to use `SimpleFsStore`");

        let root_dir = root_dir.to_owned();

        unblock(move || {
            fs::create_dir_all(&root_dir).map_err(box_error)?;

            Ok(Self {
                root_dir: root_dir.to_owned(),
            })
        })
        .await
    }

    fn file_path(&self, key: &str) -> PathBuf {
        self.root_dir.join(key)
    }
}

async fn load_file(file_path: PathBuf) -> Result<Option<Vec<u8>>, StoreError> {
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

async fn write_file(file_path: PathBuf, data: Vec<u8>) -> Result<(), StoreError> {
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

#[async_trait::async_trait]
impl Store for SimpleFsStore {
    async fn get(&self, key: &str) -> Result<Option<Node>, StoreError> {
        use borsh::BorshDeserialize;

        // Get the path to the file
        let file_path = self.file_path(key);

        if let Some(buf) = load_file(file_path).await? {
            let node = BorshNode::deserialize(&mut buf.as_slice())
                .map_err(box_error)?
                .into();

            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    async fn put(&self, key: &str, data: Node) -> Result<(), StoreError> {
        use borsh::BorshSerialize;

        // Get the path to the file
        let file_path = self.file_path(key);

        // Clone the data
        let data = BorshNode::from(data)
            .try_to_vec()
            .expect("Unreachable: IO error");

        // Write the file
        write_file(file_path, data).await
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        // Get the path to the file
        let file_path = self.file_path(key);

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
