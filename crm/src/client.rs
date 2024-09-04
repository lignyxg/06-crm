use crm::pb::crm_client::CrmClient;
use crm::pb::WelcomeRequestBuilder;
use crm::AppConfig;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load().expect("Failed to load config");
    let addr = format!("http://[::1]:{}", config.server.port);
    let mut client = CrmClient::connect(addr).await?;

    let req = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(90u32)
        .content_ids(vec![1, 2, 3])
        .build()?;

    let resp = client.welcome(req).await?.into_inner();

    println!("{:?}", resp);
    Ok(())
}
