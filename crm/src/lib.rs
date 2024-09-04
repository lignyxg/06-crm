use tonic::transport::Channel;
/// CrmService is the service
/// intended to use crm_metadata, crm_send and user_stat
use tonic::{async_trait, Request, Response, Status};

pub use config::AppConfig;
use crm_metadata::pb::metadata_client::MetadataClient;
use crm_send::pb::notification_client::NotificationClient;
use user_stat::pb::user_stats_client::UserStatsClient;

use crate::pb::crm_server::{Crm, CrmServer};
use crate::pb::{
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};

mod abi;
mod config;
pub mod pb;

#[allow(unused)]
pub struct CrmService {
    config: AppConfig,
    user_stats: UserStatsClient<Channel>,
    notification: NotificationClient<Channel>,
    metadata: MetadataClient<Channel>,
}

#[async_trait]
impl Crm for CrmService {
    async fn welcome(
        &self,
        request: Request<WelcomeRequest>,
    ) -> Result<Response<WelcomeResponse>, Status> {
        self.welcome(request.into_inner()).await
    }

    async fn recall(
        &self,
        _request: Request<RecallRequest>,
    ) -> Result<Response<RecallResponse>, Status> {
        todo!()
    }

    async fn remind(
        &self,
        _request: Request<RemindRequest>,
    ) -> Result<Response<RemindResponse>, Status> {
        todo!()
    }
}

impl CrmService {
    pub async fn new(config: AppConfig) -> Self {
        let user_stats = UserStatsClient::connect(config.server.user_stat.clone())
            .await
            .unwrap();
        let notification = NotificationClient::connect(config.server.notification.clone())
            .await
            .unwrap();
        let metadata = MetadataClient::connect(config.server.metadata.clone())
            .await
            .unwrap();
        Self {
            config,
            user_stats,
            notification,
            metadata,
        }
    }

    pub fn into_server(self) -> CrmServer<Self> {
        CrmServer::new(self)
    }
}
