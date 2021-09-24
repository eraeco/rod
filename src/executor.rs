//! Functions abstracted over the current async executor such as [`spawn`]
//!
//! Different async executors may be used on different platforms and eventually there will be Cargo
//! features for building for the desired executor.
//!
//! Currently [`smol`] will be used on native targets and [`wasm-bindgen-futures::spawn_local`]
//!
//! [`smol`]: https://docs.rs/smol
//!
//! [`wasm-bindgen-futures::spawn_local`]:
//! https://docs.rs/wasm-bindgen-futures/0.4.28/wasm_bindgen_futures/fn.spawn_local.html

pub use implementation::*;

#[cfg(not(target_arch = "wasm32"))]
mod implementation {
    use async_executor::Executor;
    use futures_lite::future;
    use once_cell::sync::Lazy;

    use std::{future::Future, panic, thread};

    /// Spawn an async task to run in the background
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) {
        static GLOBAL: Lazy<Executor<'_>> = Lazy::new(|| {
            for n in 1..=num_cpus::get() {
                thread::Builder::new()
                    .name(format!("rod-worker-{}", n))
                    .spawn(|| loop {
                        panic::catch_unwind(|| {
                            future::block_on(GLOBAL.run(future::pending::<()>()))
                        })
                        .ok();
                    })
                    .expect("cannot spawn executor thread");
            }

            Executor::new()
        });

        GLOBAL.spawn(future).detach()
    }
}

#[cfg(target_arch = "wasm32")]
mod implementation {
    use std::future::Future;

    /// Spawn an async task to run in the background
    pub fn spawn<T: Send + 'static>(_future: impl Future<Output = T> + Send + 'static) {
        todo!();
    }
}
