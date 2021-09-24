use std::process;
use tracing as trc;

use rod::engine::Rod;

use futures_lite::future;

fn main() {
    let ex = async_executor::Executor::new();
    if let Err(e) = future::block_on(ex.run(start())) {
        eprintln!("Error: {:#?}", e);
        process::exit(1);
    }
}

async fn start() -> anyhow::Result<()> {
    install_tracing();

    trc::info!("Staring server");

    let rod = &Rod::new().await?;

    {
        use rod::graph::Node;
        let mut mary = Node::new();
        mary.set("name", "Mary".to_string());

        let mut john = Node::new();
        john.set("name", "John".to_string());
        john.set("wife", &mary);

        rod.put("users/john", john).await?;
        rod.put("users/mary", mary).await?;
    }

    let node2 = dbg!(rod.get("users/john").await?).unwrap();

    dbg!(node2.get("name"));
    let wife = node2.get("wife").unwrap().follow(rod).await?.unwrap();

    dbg!(wife);

    Ok(())
}

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, fmt::format::FmtSpan, EnvFilter};

    // Build the tracing layers
    let fmt_layer =
        fmt::layer()
            .with_span_events(FmtSpan::FULL)
            .with_ansi(if atty::is(atty::Stream::Stdout) {
                true
            } else {
                false
            });
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    // Add all of the layers to the subscriber and initialize it
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
