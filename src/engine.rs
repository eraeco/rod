//! Contains the main [`Rod`] struct, used to access the replicated database

use std::sync::Arc;

use tracing as trc;

use crate::{
    graph::Node,
    store::{get_default_store, Store, StoreError},
};

/// The Rod engine, responsible for managing connections and performing the various database
/// synchronization task
///
/// [`Rod`] is the primary public API for accessing the database.
///
/// The [`Rod`] instance is cheap to clone and can be sent and shared across threads to allow
/// accessing the database concurrently from different threads.
#[derive(Clone)]
pub struct Rod {
    /// The inner data of the [`Rod`] instance
    inner: Arc<RodInner>,
}

struct RodInner {
    /// The backing data store for this engine
    store: Box<dyn Store + Sync + Send>,
}

impl Rod {
    /// Initialize a new [`Rod`] instance
    ///
    /// TODO: Use an `RodBuilder` to construct an engine with customized store and peers list
    pub async fn new() -> Result<Self, StoreError> {
        trc::trace!("Creating new Rod instance");

        // Initialize data store
        let store = Box::new(get_default_store().await?);

        // Create clonable inner data
        let inner = Arc::new(RodInner { store });

        // Create Rod instance
        let instance = Rod { inner };

        Ok(instance)
    }

    /// Get a node from the database
    pub async fn get(&self, key: &str) -> Result<Option<Node>, StoreError> {
        let this = &self.inner;

        let id = if let Some(id) = this.store.get_id(key).await?.flatten() {
            id
        } else {
            return Ok(None);
        };

        if let Some(node) = this.store.get_node(&id).await? {
            return Ok(Some(node));
        } else {
            return Ok(None);
        }
    }

    pub async fn put(&self, key: &str, node: Node) -> Result<(), StoreError> {
        let this = &self.inner;

        let id = node.id.clone();
        this.store.set_id(key, Some(id)).await?;
        this.store.put_node(node).await?;

        Ok(())
    }
}
