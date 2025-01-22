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
                .lower(to_ts(150))
                .upper(to_ts(50))
                .build()?,
        ))
        .id_builder((
            "viewed_but_not_started".to_string(),
            pb::IdQueryBuilder::default().ids(vec![270437]).build()?,
        ))
        .build()?;
    let res = client.query(req).await?.into_inner();
    let _res = res.collect::<Vec<_>>().await;
    // assert!(!res.is_empty());
    Ok(())
}
/*
 * 使用tonic进行集成测试
 * 首先启动server作为tokio的一个task，或者启动一个独立的线程也可以
 * 然后在单独的函数中调用client代码进行测试
*/
async fn start_server() -> anyhow::Result<SocketAddr> {
    let port = thread_rng().gen_range(50001..65500);
    let config = AppConfig::load().expect("Failed to load config");
    let addr = format!("[::1]:{}", port).parse()?;

    let svc = UserStatsService::new(config).await; // 1
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(svc.into_server())
            .serve(addr)
            .await
            .unwrap();
    });

    // 上文启动了一个tokio task，如果没有这句话，函数直接返回
    // 但此时这个 task 可能还没来得及被tokio调度执行
    // 如果此时有client尝试连接这个server，就会连接失败
    // 因此这里插入一个任务。
    // 在同一个上下文中，await的任务是按顺序执行的
    // 所以这里先执行 1 连接数据库，然后加入 tokio::spawn 新生成的任务
    // 然后执行这里的 sleep
    sleep(Duration::from_micros(1)).await;

    Ok(addr)
}
