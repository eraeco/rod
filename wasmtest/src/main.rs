use std::{time::Duration};

use rod::engine::Rod;

use futures_lite::future;

fn main() {
    let ex = async_executor::Executor::new();
    future::block_on(ex.run(start())).expect("Error");
}

async fn start() -> anyhow::Result<()> {
    let engine = Rod::new().await?;

    future::pending::<()>().await;

    Ok(())
}
