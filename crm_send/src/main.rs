use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as _;

use crm_send::{AppConfig, NotificationService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load().expect("Failed to load config");
    let addr = format!("[::1]:{}", config.server.port)
        .parse()
        .expect("Failed to parse address ()");
    let svc = NotificationService::new(config).into_server();
    info!("gRPC server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;
    Ok(())
}
