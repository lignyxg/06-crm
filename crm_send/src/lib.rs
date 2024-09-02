use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use tokio::sync::mpsc::Sender;
use tonic::{async_trait, Request, Response, Status, Streaming};

pub use config::AppConfig;

use crate::pb::notification_server::Notification;
use crate::pb::send_request::Msg;
use crate::pb::{SendRequest, SendResponse};

pub mod abi;
mod config;
pub mod pb;

#[derive(Clone)]
pub struct NotificationService {
    inner: Arc<NotificationServiceInner>,
}

#[allow(unused)]
pub struct NotificationServiceInner {
    config: AppConfig,
    sender_svc: Sender<Msg>,
}

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<SendResponse, tonic::Status>> + Send>>;

#[async_trait]
impl Notification for NotificationService {
    type SendStream = ResponseStream;

    async fn send(
        &self,
        request: Request<Streaming<SendRequest>>,
    ) -> ServiceResult<Self::SendStream> {
        let stream = request.into_inner();
        self.send(stream).await
    }
}
