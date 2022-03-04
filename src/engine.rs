//! Contains the main [`Rod`] struct, used to access the replicated database

use std::sync::Arc;

use tracing as trc;
use ulid::Ulid;

use crate::{
    graph::{Field, Node, Value},
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
    pub async fn get<'a, K: Into<DbIndex<'a>>>(&self, key: K) -> Result<NodeProxy, StoreError> {
        let this = &self.inner;
        let key = key.into();

        let ulid = match key {
            DbIndex::Str(s) => this.store.get_id(s).await?.flatten(),
            DbIndex::Ulid(id) => Some(id.clone()),
        };

        let id = if let Some(id) = ulid {
            id
        } else {
            return Ok(NodeProxy::new(self, Node::new()).await?);
        };

        if let Some(node) = this.store.get_node(&id).await? {
            return Ok(NodeProxy::new(self, node).await?);
        } else {
            return Ok(NodeProxy::new(self, Node::new()).await?);
        }
    }

    /// Put a node into the database
    pub async fn put<N: AsRef<Node>>(&self, key: &str, node: N) -> Result<(), StoreError> {
        let this = &self.inner;
        let new_node = node.as_ref().clone();
        let node_id = new_node.id.clone();

        let node_to_update = this.store.get_node(&new_node.id).await?;

        let new_node = if let Some(mut node) = node_to_update {
            new_node.merge_into(&mut node);
            node
        } else {
            new_node.clone()
        };

        this.store.put_node(new_node).await?;
        this.store.set_id(key, Some(node_id.clone())).await?;

        Ok(())
    }
}

/// A node loaded from the database with mutators that can be used to modify the node and
/// synchronize it back to the database
///
/// Having a [`NodeProxy`] does **not** represent exclusive access to the node data. This means that
///  there is nothing stopping another thread from modifying the node while you have a [`NodeProxy`]
pub struct NodeProxy {
    rod: Rod,
    node: Node,
}

impl AsRef<Node> for NodeProxy {
    fn as_ref(&self) -> &Node {
        &self.node
    }
}

impl std::fmt::Debug for NodeProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeProxy")
            .field("rod", &"Rod")
            .field("node", &self.node)
            .finish()
    }
}

impl NodeProxy {
    async fn new(rod: &Rod, node: Node) -> Result<Self, StoreError> {
        rod.inner.store.put_node(node.clone()).await?;

        Ok(Self {
            rod: rod.clone(),
            node,
        })
    }

    /// Get a node field
    pub fn get(&self, key: &str) -> Option<ValueRef> {
        self.node
            .fields
            .get(key)
            .map(|field| ValueRef::new(&self.rod, &field.value))
    }

    /// Set a node field
    ///
    /// > **Note:** The changes to the node are not persisted or synchronized unless you call [`Rod::put()`]
    pub fn set<V: Into<Value>>(&mut self, key: &str, value: V) {
        self.node
            .fields
            .insert(key.to_string(), Field::new(value.into()));
    }
}

impl Into<Value> for &NodeProxy {
    fn into(self) -> Value {
        Value::Node(self.node.id.clone())
    }
}

pub struct ValueRef<'a> {
    rod: Rod,
    value: &'a Value,
}

impl<'a> std::fmt::Debug for ValueRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValueRef")
            .field("rod", &"Rod")
            .field("value", &self.value)
            .finish()
    }
}

impl<'a> ValueRef<'a> {
    fn new(rod: &Rod, value: &'a Value) -> Self {
        Self {
            rod: rod.clone(),
            value,
        }
    }

    /// If this value is a reference to another node, get the node that it references from the
    /// database
    pub async fn follow(&self) -> Result<NodeProxy, StoreError> {
        let id = if let Value::Node(id) = self.value {
            id
        } else {
            return Ok(NodeProxy::new(&self.rod, Node::new()).await?);
        };

        if let Some(node) = self.rod.inner.store.get_node(id).await? {
            Ok(NodeProxy::new(&self.rod, node).await?)
        } else {
            Ok(NodeProxy::new(&self.rod, Node::new()).await?)
        }
    }

    /// Clone the referenced [`Value`] and return it
    pub fn owned(&self) -> Value {
        self.value.clone()
    }
}

impl<'a> std::ops::Deref for ValueRef<'a> {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Can be used in [`Rod::get()`], but isn't usually needed by users directly
pub enum DbIndex<'a> {
    Str(&'a str),
    Ulid(&'a Ulid),
}

impl<'a> From<&'a str> for DbIndex<'a> {
    fn from(s: &'a str) -> Self {
        DbIndex::Str(s)
    }
}

impl<'a> From<&'a Ulid> for DbIndex<'a> {
    fn from(id: &'a Ulid) -> Self {
        DbIndex::Ulid(id)
    }
}
