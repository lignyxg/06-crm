use std::ops::Deref;
use std::sync::Arc;

use chrono::Utc;
use futures::{Stream, StreamExt};
use prost_types::Timestamp;
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use tracing::{info, warn};
use uuid::Uuid;

use crate::pb::notification_server::NotificationServer;
use crate::pb::send_request::Msg;
use crate::pb::send_request::Msg::{Email, InApp, Sms};
use crate::pb::{EmailMessage, InAppMessage, SendRequest, SendResponse, SmsMessage};
use crate::{
    AppConfig, NotificationService, NotificationServiceInner, ResponseStream, ServiceResult,
};

mod email;
mod in_app;
mod sms;

const CHANNEL_SIZE: usize = 1024;

impl Deref for NotificationService {
    type Target = NotificationServiceInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl NotificationService {
    pub fn new(config: AppConfig) -> Self {
        let sender_svc = dummy_sender();
        Self {
            inner: Arc::new(NotificationServiceInner { config, sender_svc }),
        }
    }

    pub fn into_server(self) -> NotificationServer<Self> {
        NotificationServer::new(self)
    }

    pub async fn send<S>(&self, mut stream: S) -> ServiceResult<ResponseStream>
    where
        S: Stream<Item = Result<SendRequest, tonic::Status>> + Send + 'static + Unpin,
    {
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
        let svc = self.clone();
        tokio::spawn(async move {
            while let Some(Ok(req)) = stream.next().await {
                let svc_clone = svc.clone();
                let id = req.id;
                let resp = match req.msg {
                    Some(Email(email)) => email.send(id, &svc_clone).await,
                    Some(Sms(sms)) => sms.send(id, &svc_clone).await,
                    Some(InApp(in_app)) => in_app.send(id, &svc_clone).await,
                    None => {
                        warn!("Empty request");
                        Err(Status::invalid_argument("Empty request"))
                    }
                };
                if let Err(e) = tx.send(resp).await {
                    warn!("Failed to send response: {}", e);
                }
            }
        });
        // send response to client
        let resp = Box::pin(ReceiverStream::new(rx));
        Ok(Response::new(resp))
    }
}

fn dummy_sender() -> mpsc::Sender<Msg> {
    let (tx, mut rx) = mpsc::channel(CHANNEL_SIZE * 100);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            info!("Send message: {:?}", msg);
        }
    });
    tx
}

pub fn to_ts() -> Timestamp {
    let now = Utc::now();
    Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    }
}

trait Sender {
    async fn send(
        self,
        msg_id: String,
        by: &NotificationService,
    ) -> Result<SendResponse, tonic::Status>;
}

impl SendRequest {
    pub fn new_email(
        subject: impl Into<String>,
        sender: impl Into<String>,
        recipients: &[String],
        body: impl Into<String>,
    ) -> Self {
        let msg = EmailMessage {
            subject: subject.into(),
            sender: sender.into(),
            recipients: recipients.to_vec(),
            body: body.into(),
        };
        Self {
            id: Uuid::new_v4().to_string(),
            msg: Some(msg.into()),
        }
    }

    pub fn new_sms(
        sender: impl Into<String>,
        recipients: &[String],
        body: impl Into<String>,
    ) -> Self {
        let msg = SmsMessage {
            sender: sender.into(),
            recipients: recipients.to_vec(),
            body: body.into(),
        };
        Self {
            id: Uuid::new_v4().to_string(),
            msg: Some(msg.into()),
        }
    }

    pub fn new_in_app(
        device_id: impl Into<String>,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        let msg = InAppMessage {
            device_id: device_id.into(),
            title: title.into(),
            body: body.into(),
        };
        Self {
            id: Uuid::new_v4().to_string(),
            msg: Some(msg.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use tonic::codegen::tokio_stream;
    use uuid::Uuid;

    use crate::pb::{EmailMessage, InAppMessage, SendRequest, SmsMessage};
    use crate::{AppConfig, NotificationService};

    #[tokio::test]
    async fn test_send_message() -> anyhow::Result<()> {
        let svc = NotificationService::new(AppConfig::load().unwrap());
        let stream = tokio_stream::iter(
            vec![
                Ok(SendRequest {
                    id: Uuid::new_v4().to_string(),
                    msg: Some(EmailMessage::new().into()),
                }),
                Ok(SendRequest {
                    id: Uuid::new_v4().to_string(),
                    msg: Some(SmsMessage::new().into()),
                }),
                Ok(SendRequest {
                    id: Uuid::new_v4().to_string(),
                    msg: Some(InAppMessage::new().into()),
                }),
            ]
            .into_iter(),
        );
        // Send request
        let mut resp_stream = svc.send(stream).await?.into_inner();
        while let Some(Ok(res)) = resp_stream.next().await {
            println!("msg result: {:?}", res);
        }
        Ok(())
    }
}
