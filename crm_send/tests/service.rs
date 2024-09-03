use std::net::SocketAddr;
use std::time::Duration;

use futures::StreamExt;
use tokio::time::sleep;
use tonic::codegen::tokio_stream;
use tonic::Request;

use crm_send::pb::notification_client::NotificationClient;
use crm_send::pb::{EmailMessage, InAppMessage, SendRequest, SmsMessage};
use crm_send::{AppConfig, NotificationService};

#[tokio::test]
async fn send_should_work() -> anyhow::Result<()> {
    let addr = start_server().await?;
    let mut client = NotificationClient::connect(format!("http://{}", addr)).await?;
    let stream = tokio_stream::iter(
        vec![
            SendRequest {
                id: 1,
                msg: Some(EmailMessage::new().into()),
            },
            SendRequest {
                id: 2,
                msg: Some(SmsMessage::new().into()),
            },
            SendRequest {
                id: 3,
                msg: Some(InAppMessage::new().into()),
            },
        ]
        .into_iter(),
    );

    let req = Request::new(stream);
    let res = client.send(req).await?.into_inner();
    let res = res.then(|v| async { v.unwrap() }).collect::<Vec<_>>().await;

    assert_eq!(3, res.len());
    Ok(())
}

async fn start_server() -> anyhow::Result<SocketAddr> {
    let config = AppConfig::load()?;
    let addr = format!("[::1]:{}", config.server.port).parse()?;

    let svc = NotificationService::new(config);
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
