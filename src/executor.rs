use std::{future::Future, panic, thread};

use async_executor::{Executor, Task};
use futures_lite::future;
use once_cell::sync::Lazy;

pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
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

    GLOBAL.spawn(future)
}
