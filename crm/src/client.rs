use crm::pb::crm_client::CrmClient;
use crm::pb::WelcomeRequestBuilder;
use crm::AppConfig;
use tonic::metadata::MetadataValue;
use tonic::transport::Certificate;
use tonic::Request;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 得到端口号
    let config = AppConfig::load().expect("Failed to load config");
    let addr = format!("https://[::1]:{}", config.server.port);
    // 准备客户端TLS配置
    let pem = include_str!("../../fixtures/rootCA.pem");
    let tls = tonic::transport::ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(pem))
        .domain_name("localhost");
    let channel = tonic::transport::Channel::from_shared(addr.clone())?
        .timeout(std::time::Duration::from_secs(5))
        .tls_config(tls)?
        .connect()
        .await?;
    let token = include_str!("../../fixtures/token");
    let token: MetadataValue<_> = format!("Bear {token}").parse()?;

    let mut client = CrmClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token.clone());
        Ok(req)
    });

    let req = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(90u32)
        .content_ids(vec![1, 2, 3])
        .build()?;
    let resp = client.welcome(Request::new(req)).await?.into_inner();

    println!("{:?}", resp);
    Ok(())
}
