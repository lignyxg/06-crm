use tonic::Status;
use tracing::warn;

use crate::abi::{to_ts, Sender};
use crate::pb::send_request::Msg;
use crate::pb::{InAppMessage, SendResponse};
use crate::NotificationService;

impl Sender for InAppMessage {
    async fn send(
        self,
        msg_id: String,
        by: NotificationService,
    ) -> Result<SendResponse, tonic::Status> {
        by.sender_svc.send(self.into()).await.map_err(|e| {
            warn!("Failed to send message: {}", e);
            Status::internal("Failed to send message")
        })?;

        Ok(SendResponse {
            id: msg_id,
            created_at: Some(to_ts()),
        })
    }
}

impl From<InAppMessage> for Msg {
    fn from(msg: InAppMessage) -> Self {
        Msg::InApp(msg)
    }
}

#[cfg(feature = "test-util")]
impl InAppMessage {
    pub fn new() -> Self {
        use fake::faker::internet::en::MACAddress;
        use fake::faker::lorem::zh_cn::Sentence;
        use fake::Fake;

        Self {
            device_id: MACAddress().fake(),
            title: Sentence(1..2).fake(),
            body: Sentence(2..6).fake(),
        }
    }
}
