use std::{sync::Arc, time::Duration};

use tracing as trc;
use uuid::Uuid;

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

/// The interval at which modifications to the database will be flushed to disk
///
/// TODO: Make this configuable per [`Rod`] instance
const FLUSH_INTERVAL: Duration = Duration::from_secs(2);

struct RodInner {
    /// The list of nodes cached in memory
    nodes: scc::HashMap<Uuid, Node>,
    /// Nodes that have been modified in memory and need to be flushed to disk
    dirty_nodes: scc::HashMap<Uuid, ()>,
    /// The backing data store for this engine
    store: Box<dyn Store + Sync + Send>,
}

impl Rod {
    /// Initialize a new [`Rod`] instance
    ///
    /// TODO: Use an `EngineBuilder` to construct an engine with customized store and peers list
    pub async fn new() -> Result<Self, StoreError> {
        trc::trace!("Creating new instance");
        // Initialize data store
        let store = Box::new(get_default_store().await?);

        // Create clonable inner data
        let inner = Arc::new(RodInner {
            nodes: Default::default(),
            dirty_nodes: Default::default(),
            store,
        });

        // Create Rod instance
        let instance = Rod { inner };

        // Spawn a job to flush the dirty nodes to disk
        let instance_ = instance.clone();
        executor::spawn(async move {
            trc::debug!("Staring periodic node flush");
            let db = instance_.inner;

            // Loop on and interval
            let mut interval = async_timer::interval(FLUSH_INTERVAL);
            loop {
                trc::debug!("Flushing dirty nodes to disk");

                // Flush all dirty nodes to disk
                let count = db.dirty_nodes.len();
                let mut dirty_nodes = Vec::with_capacity(count);
                db.dirty_nodes.retain(|uuid, _| {
                    dirty_nodes.push(uuid.clone());
                    true
                });
                for uuid in dirty_nodes {
                    if let Some(node) = db.nodes.read(&uuid, |_, node| node.clone()) {
                        if let Err(err) = db
                            .store
                            .put_string(
                                &uuid.to_string(),
                                &serde_json::to_string_pretty(&node).expect("Serialize node"),
                            )
                            .await
                        {
                            trc::error!(%err, "Could not flush node data to the store. The store may be out-of-date!");
                        }
                    }
                }

                trc::debug!(%count, "   Flushed nodes to disk");

                interval.wait().await;
            }
        }).detach();

        Ok(instance)
    }

    // pub async fn get(key: &str) ->
}
