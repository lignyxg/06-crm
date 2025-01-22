use crate::AuthService;
use crate::config::AppConfig;
use crate::pb::{SignRequest, SignResponse, VerifyRequest, VerifyResponse};
use futures::future::BoxFuture;
use std::collections::HashSet;
use tonic::{Request, Response, Status};

use crate::pb::auth_server::AuthServer;
use jwt_simple::prelude::{
    Claims, Duration, ECDSAP256KeyPairLike, ECDSAP256PublicKeyLike, ES256KeyPair,
    VerificationOptions,
};

const JWT_DURATION: u64 = 7 * 24 * 60 * 60;
const JWT_ISSUER: &str = "crm";
const JWT_AUDIENCE: &str = "crm_client";
impl AuthService {
    pub fn try_new(config: AppConfig) -> anyhow::Result<Self> {
        let signer = ES256KeyPair::from_pem(&config.auth.sk)?;
        Ok(Self { signer, config })
    }
    pub async fn sign(
        &self,
        request: Request<SignRequest>,
    ) -> Result<Response<SignResponse>, Status> {
        let claims =
            Claims::with_custom_claims(request.into_inner(), Duration::from_secs(JWT_DURATION))
                .with_issuer(JWT_ISSUER)
                .with_audience(JWT_AUDIENCE);
        let token = self
            .signer
            .sign(claims)
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SignResponse { token }))
    }

    pub fn verify(
        &self,
        request: Request<VerifyRequest>,
    ) -> BoxFuture<'static, Result<Response<VerifyResponse>, Status>> {
        let allowed_issuers = HashSet::from([JWT_ISSUER.to_string()]);
        let allowed_audiences = HashSet::from([JWT_AUDIENCE.to_string()]);
        let opts = VerificationOptions {
            allowed_issuers: Some(allowed_issuers),
            allowed_audiences: Some(allowed_audiences),
            max_validity: Some(Duration::from_secs(JWT_DURATION)),
            ..Default::default()
        };
        let pub_key = self.signer.public_key();
        let claims = match pub_key
            .verify_token::<SignRequest>(&request.into_inner().token, Some(opts))
            .map_err(|e| Status::internal(e.to_string()))
        {
            Ok(c) => c,
            Err(e) => return Box::pin(async { Err(e) }),
        };
        let claims = claims.custom;
        Box::pin(async move {
            Ok(Response::new(VerifyResponse {
                id: claims.id,
                email: claims.email,
                name: claims.name,
                created_at: claims.created_at,
            }))
        })
    }

    pub fn into_server(self) -> AuthServer<Self> {
        AuthServer::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_sign() {
        let req = SignRequest {
            id: Uuid::new_v4().to_string(),
            email: "lily@example.com".to_string(),
            name: "Lily".to_string(),
            created_at: Some(chrono::Utc::now().into()),
        };
        let svc = AuthService::try_new(AppConfig::load().unwrap()).unwrap();
        let resp = svc.sign(Request::new(req)).await.unwrap();
        println!("resp: {:?}", resp.into_inner());
    }

    #[tokio::test]
    async fn test_verify() {
        let req = SignRequest {
            id: Uuid::new_v4().to_string(),
            email: "lily@example.com".to_string(),
            name: "Lily".to_string(),
            created_at: Some(chrono::Utc::now().into()),
        };
        let svc = AuthService::try_new(AppConfig::load().unwrap()).unwrap();
        let resp = svc.sign(Request::new(req)).await.unwrap().into_inner();

        let req = VerifyRequest { token: resp.token };
        let svc = AuthService::try_new(AppConfig::load().unwrap()).unwrap();
        let resp = svc.verify(Request::new(req)).await.unwrap();
        println!("resp: {:?}", resp.into_inner());
    }
}
