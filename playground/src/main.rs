use std::{process, time::Duration};
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

    let rod = Rod::new().await?;

    // {
    //     use rod::{
    //         graph::{Field, Node, Value},
    //         Ulid,
    //     };
    //     let node1 = Node::new();
    //     let node2 = Node::new_with_fields(vec![
    //         ("hello".into(), Field::new(Value::String("world".into()))),
    //         (
    //             "someJunk".into(),
    //             Field::new(Value::Binary(vec![1, 2, 3, 4])),
    //         ),
    //         ("age".into(), Field::new(Value::Float(30.0))),
    //         ("nothing".into(), Field::new(Value::None)),
    //         ("anotherNode".into(), Field::new(Value::Node(Ulid::new()))),
    //     ]);

    //     rod.put("node1", node1).await?;
    //     rod.put("node2", node2).await?;
    // }

    dbg!(rod.get("node2").await?);

    // Just prevent the process from exiting
    let mut interval = async_timer::interval(Duration::from_secs(1));
    loop {
        interval.wait().await;
    }
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
