use std::{pin::Pin, sync::Arc};

use futures::Stream;
use sqlx::PgPool;
use tonic::{async_trait, Request, Response, Status};

pub use config::AppConfig;

use crate::pb::user_stats_server::UserStats;
use crate::pb::{QueryRequest, RawQueryRequest, User};

pub mod abi;
mod config;
pub mod pb;

#[derive(Clone)]
pub struct UserStatsService {
    inner: Arc<UserStatsServiceInner>,
}

#[allow(unused)]
pub struct UserStatsServiceInner {
    config: AppConfig,
    pool: PgPool,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, tonic::Status>> + Send>>;

#[async_trait]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;

    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        self.query(request.into_inner()).await
    }

    type RawQueryStream = ResponseStream;

    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        self.raw_query(request.into_inner()).await
    }
}
