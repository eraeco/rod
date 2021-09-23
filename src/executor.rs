pub use implementation::*;

#[cfg(not(target_arch = "wasm32"))]
mod implementation {
    use async_executor::Executor;
    use futures_lite::future;
    use once_cell::sync::Lazy;

    use std::{future::Future, panic, thread};

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

    pub fn spawn<T: Send + 'static>(_future: impl Future<Output = T> + Send + 'static) {
        todo!();
    }
}
