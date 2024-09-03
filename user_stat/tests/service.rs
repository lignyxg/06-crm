use rand::{thread_rng, Rng};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;
use tonic::codegen::tokio_stream::StreamExt;

use user_stat::pb::user_stats_client::UserStatsClient;
use user_stat::test_util::to_ts;
use user_stat::{pb, AppConfig, UserStatsService};

#[tokio::test]
async fn raw_query_should_work() -> anyhow::Result<()> {
    let addr = start_server().await?;
    let mut client = UserStatsClient::connect(format!("http://{}", addr)).await?;

    let req = pb::RawQueryRequestBuilder::default()
        .query("SELECT email, name FROM user_stats WHERE created_at BETWEEN '2024-05-01 00:00:00' AND '2024-08-02 00:00:00' AND array[270437] <@ viewed_but_not_started limit 5".to_string())
        .build()?;
    let res = client.raw_query(req).await?.into_inner();
    let res = res.collect::<Vec<_>>().await;
    assert!(!res.is_empty());
    Ok(())
}

#[tokio::test]
async fn query_should_work() -> anyhow::Result<()> {
    let addr = start_server().await?;
    let mut client = UserStatsClient::connect(format!("http://{}", addr)).await?;

    let req = pb::QueryRequestBuilder::default()
        .timestamp_builder((
            "created_at".to_string(),
            pb::TimeQueryBuilder::default()
                .lower(to_ts(100))
                .upper(to_ts(20))
                .build()?,
        ))
        .id_builder((
            "viewed_but_not_started".to_string(),
            pb::IdQueryBuilder::default().ids(vec![270437]).build()?,
        ))
        .build()?;
    let res = client.query(req).await?.into_inner();
    let res = res.collect::<Vec<_>>().await;
    assert!(!res.is_empty());
    Ok(())
}

async fn start_server() -> anyhow::Result<SocketAddr> {
    let port = thread_rng().gen_range(50001..65500);
    let config = AppConfig::load().expect("Failed to load config");
    let addr = format!("[::1]:{}", port).parse()?;

    let svc = UserStatsService::new(config).await;
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
