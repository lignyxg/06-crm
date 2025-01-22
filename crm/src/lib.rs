use crate::abi::auth_interceptor::AuthInterceptor;
use crate::pb::crm_server::{Crm, CrmServer};
use crate::pb::{
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};
use crate::worker_thread::WorkerThread;
use auth::pb::auth_client::AuthClient;
pub use config::AppConfig;
use crm_metadata::pb::metadata_client::MetadataClient;
use crm_send::pb::notification_client::NotificationClient;
use std::sync::{mpsc, Arc, Mutex};
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;
/// CrmService is the service
/// intended to use crm_metadata, crm_send and user_stat
use tonic::{async_trait, Request, Response, Status};
use user_stat::pb::user_stats_client::UserStatsClient;

mod abi;
pub mod config;
pub mod pb;
pub mod worker_thread;

#[allow(unused)]
pub struct CrmService {
    config: AppConfig,
    user_stats: UserStatsClient<Channel>,
    notification: NotificationClient<Channel>,
    metadata: MetadataClient<Channel>,
    auth: AuthClient<Channel>,
    worker_thread: WorkerThread,
    request_tx: mpsc::Sender<Request<()>>,
    result_rx: Arc<Mutex<mpsc::Receiver<Request<()>>>>,
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
        let auth = AuthClient::connect(config.server.auth.clone())
            .await
            .unwrap();
        let mut worker_thread = WorkerThread::new(auth.clone());
        let (req_tx, req_rx) = mpsc::channel();
        let res_rx = worker_thread.setup(req_rx);
        Self {
            config,
            user_stats,
            notification,
            metadata,
            auth,
            worker_thread,
            request_tx: req_tx,
            result_rx: Arc::new(Mutex::new(res_rx)),
        }
    }

    // pub fn into_server(self) -> InterceptedService<CrmServer<CrmService>, WorkerThread> {
    //     let worker = WorkerThread::new(self.auth.clone());
    //     CrmServer::with_interceptor(self, worker)
    // }

    pub fn into_server(self) -> InterceptedService<CrmServer<CrmService>, AuthInterceptor> {
        let worker = AuthInterceptor::new(self.request_tx.clone(), self.result_rx.clone());
        CrmServer::with_interceptor(self, worker)
    }
}
