use auth::config::AppConfig;
use auth::pb::SignRequest;
use auth::pb::auth_client::AuthClient;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load().expect("Failed to load config");
    let addr = format!("http://[::1]:{ }", config.server.port);

    let mut client = AuthClient::connect(addr).await?;
    let req = SignRequest {
        id: Uuid::new_v4().to_string(),
        email: "lily@example.com".to_string(),
        name: "Lily".to_string(),
        created_at: Some(chrono::Utc::now().into()),
    };

    let resp = client.sign(req).await?;
    println!("resp: {:?}", resp.into_inner());
    Ok(())
}
