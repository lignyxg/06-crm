use crm::pb::user_service_server::{UserService, UserServiceServer};
use crm::pb::{CreateUserRequest, GetUserRequest, User};
use tonic::{async_trait, Request, Response, Status};

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
    let addr = "[::1]:50051".parse().unwrap();
    let user = UserServer;
    println!("gRPC server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(UserServiceServer::new(user))
        .serve(addr)
        .await?;
    Ok(())
}
