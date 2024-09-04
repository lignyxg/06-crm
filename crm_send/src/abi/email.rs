use tracing::warn;

use crate::abi::{to_ts, Sender};
use crate::pb::send_request::Msg;
use crate::pb::{EmailMessage, SendResponse};
use crate::NotificationService;

impl Sender for EmailMessage {
    async fn send(
        self,
        msg_id: String,
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

impl From<EmailMessage> for Msg {
    fn from(msg: EmailMessage) -> Self {
        Msg::Email(msg)
    }
}

#[cfg(feature = "test-util")]
impl EmailMessage {
    pub fn new() -> Self {
        use fake::faker::internet::en::SafeEmail;
        use fake::faker::lorem::zh_cn::Sentence;
        use fake::faker::name::en::Name;
        use fake::Fake;
        use rand::{thread_rng, Rng};

        let count = thread_rng().gen_range(1..5);
        let recipients = (1..count).map(|_| SafeEmail().fake()).collect::<Vec<_>>();
        Self {
            subject: Sentence(1..2).fake(),
            sender: Name().fake(),
            recipients,
            body: Sentence(2..6).fake(),
        }
    }
}
