use crm::pb::user_service_client::UserServiceClient;
use crm::pb::CreateUserRequest;
use tonic::Request;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;

    let request = Request::new(CreateUserRequest {
        name: "John".to_string(),
        email: "j@j.com".to_string(),
    });

    let response = client.create_user(request).await?.into_inner();
    println!("RESPONSE={:?}", response);
    Ok(())
}
