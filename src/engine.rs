use std::{sync::Arc, time::Duration};

use tracing as trc;
use ulid::Ulid;

use crate::{
    executor,
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
    /// The list of nodes cached in memory
    nodes: scc::HashMap<Ulid, Node>,
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
        let inner = Arc::new(RodInner {
            nodes: Default::default(),
            store,
        });

        // Create Rod instance
        let instance = Rod { inner };

        Ok(instance)
    }

    // pub async fn get(&self, key: &str) -> Value {
    //     self.inner.store.
    // }
}
