use tonic::codegen::tokio_stream::StreamExt;
use tonic::{Response, Status};

use crm_metadata::pb::MaterializeRequest;
use crm_send::pb::SendRequest;
use user_stat::pb::QueryRequest;

use crate::pb::{WelcomeRequest, WelcomeResponse};
use crate::CrmService;

const ADMIN: &str = "admin@crm.org";

impl CrmService {
    pub async fn welcome(
        &self,
        request: WelcomeRequest,
    ) -> Result<Response<WelcomeResponse>, Status> {
        let mut user_stats_client = self.user_stats.clone();
        let users = user_stats_client
            .query(QueryRequest::new_with_interval(request.interval))
            .await?
            .into_inner();

        let contents = self
            .metadata
            .clone()
            .materialize(MaterializeRequest::new_with_ids(&request.content_ids))
            .await?
            .into_inner();

        let contents = contents.filter_map(|c| c.ok()).collect::<Vec<_>>().await;

        let mut notify = self.notification.clone();

        let req = users.filter_map(move |user| {
            let user = user.ok()?;
            Some(SendRequest::new_email(
                "Welcome".to_string(),
                ADMIN.to_string(),
                &[user.email],
                format!(
                    "Hi, {}! Welcome to CRM! \nContents for you: {:?}",
                    user.name, &contents
                ),
            ))
        });

        notify.send(req).await?;

        let resp = WelcomeResponse { id: request.id };
        Ok(Response::new(resp))
    }
}
