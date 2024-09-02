use std::pin::Pin;

use futures::Stream;
use tonic::{async_trait, Request, Response, Status, Streaming};

pub use config::AppConfig;

use crate::pb::metadata_server::{Metadata, MetadataServer};
use crate::pb::{Content, MaterializeRequest};

mod abi;
mod config;
pub mod pb;

#[allow(unused)]
pub struct MetadataService {
    config: AppConfig,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Content, tonic::Status>> + Send>>;
#[async_trait]
impl Metadata for MetadataService {
    type MaterializeStream = ResponseStream;

    async fn materialize(
        &self,
        request: Request<Streaming<MaterializeRequest>>,
    ) -> ServiceResult<Self::MaterializeStream> {
        let req = request.into_inner();
        self.materialize(req).await
    }
}

impl MetadataService {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }

    pub fn into_server(self) -> MetadataServer<Self> {
        MetadataServer::new(self)
    }
}
