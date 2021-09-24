//! Store implementatons for native targets

use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use blocking::unblock;
use borsh::BorshDeserialize;
use ulid::Ulid;

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
    /// The directory to store nodes in
    node_dir: PathBuf,
    /// The directory to store id mappings in
    id_dir: PathBuf,
}

impl SimpleFsStore {
    /// Create a new [`SimpleFsStore`] that puts files in the specified `root_dir`
    pub async fn new(root_dir: &Path) -> Result<SimpleFsStore, StoreError> {
        #[cfg(not(feature = "borsh"))]
        compile_error!("`borsh` feature required to use `SimpleFsStore`");

        let root_dir = root_dir.to_owned();

        unblock(move || {
            let store = Self {
                node_dir: root_dir.join("nodes"),
                id_dir: root_dir.join("ids"),
            };

            fs::create_dir_all(&store.node_dir).boxed_err()?;
            fs::create_dir_all(&store.id_dir).boxed_err()?;

            Ok(store)
        })
        .await
    }

    fn node_path(&self, key: &Ulid) -> PathBuf {
        self.node_dir.join(key.to_string())
    }

    fn id_path(&self, key: &str) -> PathBuf {
        self.id_dir.join(base64::encode(key))
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
        let mut file = OpenOptions::new().read(true).open(file_path).boxed_err()?;

        // Read the file into buffer
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).boxed_err()?;

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
            .boxed_err()?;

        // Write the data to the file
        Ok(file.write_all(&data).boxed_err()?)
    })
    .await
}

#[async_trait::async_trait]
impl Store for SimpleFsStore {
    async fn get_node(&self, key: &Ulid) -> Result<Option<Node>, StoreError> {
        use borsh::BorshDeserialize;

        // Get the path to the file
        let file_path = self.node_path(key);

        if let Some(buf) = load_file(file_path).await? {
            let node = BorshNode::deserialize(&mut buf.as_slice())
                .boxed_err()?
                .into();

            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    async fn put_node(&self, node: Node) -> Result<(), StoreError> {
        use borsh::BorshSerialize;

        // Get the path to the file
        let file_path = self.node_path(&node.id);

        // Clone the data
        let data = BorshNode::from(node)
            .try_to_vec()
            .expect("Unreachable: IO error");

        // Write the file
        write_file(file_path, data).await
    }

    async fn delete_node(&self, key: &Ulid) -> Result<(), StoreError> {
        // Get the path to the file
        let file_path = self.node_path(key);

        // Perform blocking operation on a thread pool
        unblock(move || {
            // Delete the file
            fs::remove_file(file_path).boxed_err()?;

            // Write the data to the file
            Ok(())
        })
        .await
    }

    async fn set_id(&self, key: &str, id: Option<Ulid>) -> Result<(), StoreError> {
        let file_path = self.id_path(key);
        let data = borsh::to_vec(&id.map(|x| u128::from(x))).boxed_err()?;

        write_file(file_path, data).await
    }

    async fn get_id(&self, key: &str) -> Result<Option<Option<Ulid>>, StoreError> {
        let file_path = self.id_path(key);
        if let Some(buf) = load_file(file_path).await? {
            let id = Option::<u128>::deserialize(&mut buf.as_slice())
                .boxed_err()?
                .map(Ulid::from);

            Ok(Some(id))
        } else {
            Ok(None)
        }
    }
}

trait BoxedError<T, E> {
    fn boxed_err(self) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> BoxedError<T, E> for Result<T, E> {
    fn boxed_err(self) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.map_err(box_error)
    }
}

/// Helper to box the an error
fn box_error(
    e: impl std::error::Error + Sync + Send + 'static,
) -> Box<dyn std::error::Error + Sync + Send> {
    Box::new(e)
}
