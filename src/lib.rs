pub mod engine;
pub mod graph;
pub mod protocol;
pub mod store;
pub mod merge;

pub use ulid::Ulid;

pub(crate) mod executor;

#[cfg(target_arch = "wasm32")]
extern crate wee_alloc;

// Use `wee_alloc` as the global allocator for WASM
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
