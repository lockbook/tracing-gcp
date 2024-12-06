use std::fs;

use tracing::info;
use tracing_appender::rolling::{self};
use tracing_gcp::GcpEventFormatter;
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry};
use uuid::Uuid;

#[test]
fn test_setup() {
    let log = Uuid::new_v4().to_string();
    let subscriber = Registry::default().with(
        fmt::Layer::new()
            .event_format(GcpEventFormatter::default())
            .with_writer(rolling::never("/tmp", &log)),
    );

    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("test");
    info!(a = 5, "test");

    let log_content = fs::read_to_string(format!("/tmp/{log}")).unwrap();

    println!("{log_content}");
    assert!(log_content.contains("test"));
}
