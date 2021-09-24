//! The Rust implementation of the [GUN] decentralized database syncronization protocol.
//!
//! Rod is attempting to be compatible with the official JavaScript implementation of GUN while also
//! supporting extra features such as a binary serialization
//!
//! [GUN]: https://github.com/amark/gun

pub mod crdt;
pub mod engine;
pub mod executor;
pub mod graph;
pub mod protocol;
pub mod store;

pub use ulid::Ulid;
