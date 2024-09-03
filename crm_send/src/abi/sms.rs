use tracing::warn;

use crate::abi::{to_ts, Sender};
use crate::pb::send_request::Msg;
use crate::pb::{SendResponse, SmsMessage};
use crate::NotificationService;

impl Sender for SmsMessage {
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

impl From<SmsMessage> for Msg {
    fn from(msg: SmsMessage) -> Self {
        Msg::Sms(msg)
    }
}

#[cfg(feature = "test-util")]
impl SmsMessage {
    pub fn new() -> Self {
        use fake::faker::lorem::zh_cn::Sentence;
        use fake::faker::phone_number::zh_cn::PhoneNumber;
        use fake::Fake;
        use rand::{thread_rng, Rng};

        let count = thread_rng().gen_range(0..3);
        let recipients = (0..count).map(|_| PhoneNumber().fake()).collect();
        Self {
            sender: PhoneNumber().fake(),
            recipients,
            body: Sentence(1..3).fake(),
        }
    }
}
