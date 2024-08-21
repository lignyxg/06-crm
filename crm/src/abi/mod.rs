use crate::pb::User;
use prost_types::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH};

impl User {
    pub fn new(name: &str, email: &str, password: &str) -> User {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let timestamp = Timestamp {
            seconds: now.as_secs() as i64,
            nanos: now.subsec_nanos() as i32,
        };
        User {
            name: name.to_string(),
            email: email.to_string(),
            password: password.to_string(),
            created_at: Some(timestamp),
        }
    }
}
