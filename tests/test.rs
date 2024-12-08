use std::fs::{self, File};
use tracing::*;
use tracing_gcp::GcpLayer;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use uuid::Uuid;

#[test]
fn log_readout() {
    let log = Uuid::new_v4().to_string();
    let log = format!("/tmp/{log}");
    println!("log file: {log}");

    let subscriber = Registry::default().with(GcpLayer::init_with_writer(
        File::create(log.clone()).unwrap(),
    ));

    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("test");
    warn!(a = 5, "test");
    info!(requestMethod = "post", "req received");

    let log_content = fs::read_to_string(log).unwrap();

    println!("{log_content}");
    assert!(log_content.contains("test"));
}
