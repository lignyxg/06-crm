use std::net::SocketAddr;
use std::time::Duration;

use futures::TryStreamExt;
use tokio::time::sleep;
use tonic::codegen::tokio_stream;

use crm_metadata::pb::metadata_client::MetadataClient;
use crm_metadata::pb::MaterializeRequest;
use crm_metadata::{AppConfig, MetadataService};

#[tokio::test]
async fn test_metadata() -> anyhow::Result<()> {
    let addr = start_server().await?;
    let mut client = MetadataClient::connect(format!("http://{}", addr)).await?;

    let stream = tokio_stream::iter(vec![
        MaterializeRequest { id: 1 },
        MaterializeRequest { id: 2 },
    ]);

    // let req = Request::new(stream);
    let res = client.materialize(stream).await?.into_inner();
    let res = res.try_collect::<Vec<_>>().await?;

    assert_eq!(2, res.len());
    Ok(())
}

async fn start_server() -> anyhow::Result<SocketAddr> {
    let config = AppConfig::load()?;
    let addr = format!("[::1]:{}", config.server.port).parse()?;

    let svc = MetadataService::new(config);
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(svc.into_server())
            .serve(addr)
            .await
            .unwrap();
    });

    sleep(Duration::from_micros(1)).await;

    Ok(addr)
}
