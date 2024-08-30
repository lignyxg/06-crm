use std::ops::Deref;
use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use itertools::Itertools;
use prost_types::Timestamp;
use sqlx::PgPool;
use tonic::Response;

use crate::pb::user_stats_server::UserStatsServer;
use crate::pb::{IdQuery, QueryRequest, RawQueryRequest, TimeQuery, User};
use crate::{AppConfig, ResponseStream, ServiceResult, UserStatsService, UserStatsServiceInner};

impl UserStatsService {
    pub async fn query(&self, req: QueryRequest) -> ServiceResult<ResponseStream> {
        // generate sql based on request
        let sql = Self::query_sql(req);
        let Ok(ret) = sqlx::query_as::<_, User>(&sql).fetch_all(&self.pool).await else {
            return Err(tonic::Status::internal(format!(
                "Failed to query with: {}",
                sql
            )));
        };

        let rep_stream = Box::pin(futures::stream::iter(ret.into_iter().map(Ok)));
        Ok(Response::new(rep_stream))
    }

    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.pool)
            .await
        else {
            return Err(tonic::Status::internal(format!(
                "Failed to query with: {}",
                req.query
            )));
        };

        let rep_stream = Box::pin(futures::stream::iter(ret.into_iter().map(Ok)));

        Ok(Response::new(rep_stream))
    }

    pub fn query_sql(req: QueryRequest) -> String {
        let mut sql = "SELECT email, name FROM user_stats WHERE ".to_string();

        let time_conds = req
            .timestamps
            .iter()
            .map(|(k, v)| format!("{} {} ", k, time_query(v)))
            .join(" AND ");

        sql.push_str(&time_conds);
        sql.push_str(" AND ");

        let id_conds = req.ids.iter().map(|(k, v)| id_query(k, v)).join(" AND ");

        sql.push_str(&id_conds);

        sql
    }

    pub async fn new(config: AppConfig) -> Self {
        let pool = PgPool::connect(&config.db_url)
            .await
            .expect("Failed to connect to db");
        let inner = UserStatsServiceInner { config, pool };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}

impl Deref for UserStatsService {
    type Target = UserStatsServiceInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn time_query(v: &TimeQuery) -> String {
    let (lower, upper) = (v.lower, v.upper);
    if lower.is_none() && upper.is_none() {
        return "TRUE".to_string();
    }
    if upper.is_none() {
        let t = to_sqlx_timestamp(lower.unwrap());
        return format!(" >= '{}' ", t.to_rfc3339());
    }

    if lower.is_none() {
        let t = to_sqlx_timestamp(upper.unwrap());
        return format!(" <= '{}' ", t.to_rfc3339());
    }

    let t_lower = to_sqlx_timestamp(lower.unwrap());
    let t_upper = to_sqlx_timestamp(upper.unwrap());
    format!(
        " BETWEEN '{}' AND '{}' ",
        t_lower.to_rfc3339(),
        t_upper.to_rfc3339()
    )
}

fn to_sqlx_timestamp(t: Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(t.seconds, t.nanos as u32).unwrap()
}

fn id_query(k: &str, v: &IdQuery) -> String {
    if v.ids.is_empty() {
        return "TRUE".to_string();
    }
    format!("array{:?} <@ {}", v.ids, k)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::{Days, Utc};
    use futures::StreamExt;
    use prost_types::Timestamp;

    use crate::{pb, AppConfig, UserStatsService};

    // #[test]
    // fn test_query_sql() -> Result<()> {
    //     let req = pb::QueryRequestBuilder::default()
    //         .timestamp_builder((
    //             "created_at".to_string(),
    //             pb::TimeQueryBuilder::default()
    //                 .lower(to_ts(10))
    //                 .upper(to_ts(20))
    //                 .build()?,
    //         ))
    //         .id_builder((
    //             "viewed_but_not_started".to_string(),
    //             pb::IdQueryBuilder::default().ids(vec![1, 2]).build()?,
    //         ))
    //         .build()?;
    //     let sql = UserStatsService::query_sql(req);
    //     println!("sql: {}", sql);
    //     Ok(())
    // }

    fn to_ts(days: u64) -> Timestamp {
        let now = Utc::now().checked_sub_days(Days::new(days)).unwrap();
        Timestamp {
            seconds: now.timestamp(),
            nanos: now.timestamp_subsec_nanos() as i32,
        }
    }

    #[tokio::test]
    async fn test_raw_query() -> Result<()> {
        let config = AppConfig::load().expect("Failed to load config");
        let svc = UserStatsService::new(config).await;

        let raw_sql = "SELECT email, name FROM user_stats \
        WHERE created_at BETWEEN '2024-05-01 00:00:00' AND '2024-08-02 00:00:00' \
        AND array[270437] <@ viewed_but_not_started limit 5";

        let req = pb::RawQueryRequestBuilder::default()
            .query(raw_sql.to_string())
            .build()?;
        let mut ret = svc.raw_query(req).await?.into_inner();
        while let Some(user) = ret.next().await {
            println!("user: {:?}", user?);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_query() -> Result<()> {
        let config = AppConfig::load().expect("Failed to load config");
        let svc = UserStatsService::new(config).await;

        let req = pb::QueryRequestBuilder::default()
            .timestamp_builder((
                "created_at".to_string(),
                pb::TimeQueryBuilder::default()
                    .lower(to_ts(120))
                    .upper(to_ts(20))
                    .build()?,
            ))
            .id_builder((
                "viewed_but_not_started".to_string(),
                pb::IdQueryBuilder::default().ids(vec![270437]).build()?,
            ))
            .build()?;
        let mut ret = svc.query(req).await?.into_inner();
        while let Some(user) = ret.next().await {
            println!("user: {:?}", user?);
        }

        Ok(())
    }
}
