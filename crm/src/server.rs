use tonic::{async_trait, Request, Response, Status};
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer as _;

use crm::pb::user_service_server::{UserService, UserServiceServer};
use crm::pb::{CreateUserRequest, GetUserRequest, User};

#[derive(Default)]
pub struct UserServer;

#[async_trait]
impl UserService for UserServer {
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("Received: {:?}", input);
        Ok(Response::new(User::default()))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("Received: {:?}", input);
        let user = User::new(&input.name, &input.email, "123456");
        Ok(Response::new(user))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "[::1]:50051".parse().unwrap();
    let srv = UserServer;
    info!("gRPC server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(UserServiceServer::new(srv))
        .serve(addr)
        .await?;
    Ok(())
}
