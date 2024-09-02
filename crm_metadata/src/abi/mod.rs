use chrono::{DateTime, Days, Utc};
use fake::faker::chrono::en::DateTimeBetween;
use fake::faker::internet::en::DomainSuffix;
use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::Name;
use fake::{Fake, Faker};
use futures::{Stream, StreamExt};
use prost_types::Timestamp;
use rand::{thread_rng, Rng};
use tokio::sync::mpsc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::Response;

use crate::pb::{Content, MaterializeRequest, Publisher};
use crate::{MetadataService, ResponseStream, ServiceResult};

const CHANNEL_SIZE: usize = 1024;
impl MetadataService {
    // it's hard to construct a tonic::Streaming request for test
    // so use generic stream instead
    pub async fn materialize<S>(&self, mut stream: S) -> ServiceResult<ResponseStream>
    where
        S: Stream<Item = Result<MaterializeRequest, tonic::Status>> + Send + 'static + Unpin,
    {
        let (tx, rx) = mpsc::channel::<Result<Content, tonic::Status>>(CHANNEL_SIZE);

        tokio::spawn(async move {
            while let Some(Ok(req)) = stream.next().await {
                // get request id from client
                let id = req.id;
                // generate dummy content
                let content = Content::new(id);
                // send to client
                tx.send(Ok(content)).await.unwrap();
            }
        });

        let ret_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(ret_stream)))
    }
}

impl Content {
    pub fn new(id: u32) -> Self {
        let psize = thread_rng().gen_range(1..5);
        let mut publishers = Vec::with_capacity(psize);
        for _ in 0..psize {
            publishers.push(Publisher::new());
        }
        Self {
            id,
            name: Name().fake(),
            description: Sentence(1..3).fake(),
            publishers,
            url: DomainSuffix().fake(),
            image: "https://placehold.co/1600x900".to_string(),
            content_type: Faker.fake(),
            created_at: Some(timestamp()),
            views: (1000..1000000).fake(),
            likes: (5..100000).fake(),
            dislikes: (5..100000).fake(),
        }
    }
}

fn before(days: u64) -> DateTime<Utc> {
    Utc::now().checked_sub_days(Days::new(days)).unwrap()
}

fn timestamp() -> Timestamp {
    let fake_ts: DateTime<Utc> = DateTimeBetween(before(120), before(1)).fake();
    Timestamp {
        seconds: fake_ts.timestamp(),
        nanos: fake_ts.timestamp_subsec_nanos() as i32,
    }
}

impl Publisher {
    pub fn new() -> Self {
        Self {
            id: (100..1000000).fake(),
            name: Name().fake(),
            avatar: "https://placehold.co/600x600".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tonic::codegen::tokio_stream;

    use crate::AppConfig;

    use super::*;

    #[tokio::test]
    async fn test_materialize() -> Result<()> {
        let svc = MetadataService::new(AppConfig::load().unwrap());
        let stream = tokio_stream::iter(
            vec![
                Ok(MaterializeRequest { id: 1 }),
                Ok(MaterializeRequest { id: 2 }),
            ]
            .into_iter(),
        );
        // send request
        let mut resp_stream = svc.materialize(stream).await?.into_inner();
        while let Some(Ok(material)) = resp_stream.next().await {
            println!("material: {:#?}", material);
        }
        Ok(())
    }
}
