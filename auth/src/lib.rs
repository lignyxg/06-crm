use crate::config::AppConfig;
use crate::pb::auth_server::Auth;
use crate::pb::{SignRequest, SignResponse, VerifyRequest, VerifyResponse};
use jwt_simple::prelude::ES256KeyPair;
use tonic::{Request, Response, Status, async_trait};

mod abi;
pub mod config;
pub mod pb;

#[allow(unused)]
pub struct AuthService {
    signer: ES256KeyPair,
    config: AppConfig,
}

#[async_trait]
impl Auth for AuthService {
    async fn sign(&self, request: Request<SignRequest>) -> Result<Response<SignResponse>, Status> {
        self.sign(request).await
    }

    async fn verify(
        &self,
        request: Request<VerifyRequest>,
    ) -> Result<Response<VerifyResponse>, Status> {
        println!("verify req: {:?}", request);
        self.verify(request).await
    }
}
