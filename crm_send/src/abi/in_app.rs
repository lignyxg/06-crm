use tracing::warn;

use crate::abi::{to_ts, Sender};
use crate::pb::send_request::Msg;
use crate::pb::{InAppMessage, SendResponse};
use crate::NotificationService;

impl Sender for InAppMessage {
    async fn send(
        self,
        msg_id: u32,
        by: &NotificationService,
    ) -> Result<SendResponse, tonic::Status> {
        let snd = by.sender_svc.clone();
        if let Err(e) = snd.send(self.into()).await {
            warn!("Failed to send email: {}", e);
        };

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
