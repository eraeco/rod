use std::process;
use tracing as trc;

use rod::prelude::*;

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

    let mary = rod
        .get("users/mary")
        .await?
        .tap_mut(|x| x.set("name", "Mary".to_string()))
        .tap_mut(|x| x.set("age", 32));
    rod.put("users/mary", &mary).await?;

    rod.get("users/john")
        .await?
        .tap_mut(|x| x.set("name", "John".to_string()))
        .tap_mut(|x| x.set("wife", &mary))
        .pipe(|x| rod.put("users/john", x))
        .await?;

    let wife_name = rod
        .get("users/john")
        .await?
        .get("wife")
        .unwrap()
        .follow()
        .await?
        .get("name")
        .unwrap()
        .owned();

    dbg!(wife_name);

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
