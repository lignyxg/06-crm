use tonic::transport::{Identity, ServerTlsConfig};
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as _;

use crm::{AppConfig, CrmService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(layer).init();

    let mut config = AppConfig::load().expect("Failed to load config");
    let addr = format!("[::1]:{}", config.server.port)
        .parse()
        .expect("Failed to parse address");

    let tls_config = config.server.tls.take(); // 如果开启了TLS,则保存TLS配置
                                               // let tls_config: Option<TlsConfig> = None;

    let svc = CrmService::new(config).await.into_server();
    info!("Crm service listening on {}", addr);

    if let Some(tls) = tls_config {
        // 如果开启了TLS
        let identity = Identity::from_pem(tls.cert, tls.key);
        tonic::transport::Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .add_service(svc)
            .serve(addr)
            .await?;
    } else {
        tonic::transport::Server::builder()
            .add_service(svc)
            .serve(addr)
            .await?;
    }
    Ok(())
}
