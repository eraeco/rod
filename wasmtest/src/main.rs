use rod::engine::Rod;

fn main() {
    pollster::block_on(start());
}

async fn start() {
    let _engine = Rod::new().await.unwrap();

    std::future::pending::<()>().await;
}
