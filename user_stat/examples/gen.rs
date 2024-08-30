use std::collections::HashSet;
use std::hash::Hash;

use chrono::{DateTime, Days, Utc};
use fake::faker::chrono::fr_fr::DateTimeBetween;
use fake::{
    faker::{internet::en::SafeEmail, name::zh_cn::Name},
    Dummy, Fake, Faker,
};
use nanoid::nanoid;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, PgPool};

#[derive(Debug, Dummy, Serialize, Deserialize, Eq, PartialEq)]
struct UserStat {
    #[dummy(faker = "UniqueEmail")]
    pub email: String,
    #[dummy(faker = "Name()")]
    pub name: String,
    pub gender: Gender,
    #[dummy(faker = "DateTimeBetween(before(365*5), before(90))")]
    pub created_at: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(30), now())")]
    pub last_visited_at: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(90), now())")]
    pub last_watched_at: DateTime<Utc>,
    #[dummy(faker = "InitList(50, 100_000, 100_000)")]
    pub recent_watched: Vec<i32>,
    #[dummy(faker = "InitList(50, 200_000, 100_000)")]
    pub viewed_but_not_started: Vec<i32>,
    #[dummy(faker = "InitList(50, 300_000, 100_000)")]
    pub started_but_not_finished: Vec<i32>,
    #[dummy(faker = "InitList(50, 400_000, 100_000)")]
    pub finished: Vec<i32>,
    #[dummy(faker = "DateTimeBetween(before(45), now())")]
    pub last_email_notification: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(30), now())")]
    pub last_in_app_notification: DateTime<Utc>,
    #[dummy(faker = "DateTimeBetween(before(30), now())")]
    pub last_sms_notification: DateTime<Utc>,
}

#[derive(Debug, Dummy, Serialize, Deserialize, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "gender", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

impl Hash for UserStat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.email.hash(state);
    }
}

fn before(days: u64) -> DateTime<Utc> {
    Utc::now().checked_sub_days(Days::new(days)).unwrap()
}
fn now() -> DateTime<Utc> {
    Utc::now()
}

struct InitList(pub i32, pub i32, pub i32);

impl Dummy<InitList> for Vec<i32> {
    fn dummy_with_rng<R: Rng + ?Sized>(v: &InitList, rng: &mut R) -> Vec<i32> {
        let (max, start, len) = (v.0, v.1, v.2);
        let size = rng.gen_range(0..max);
        (0..size)
            .map(|_| rng.gen_range(start..(start + len)))
            .collect()
    }
}

struct UniqueEmail;

const ALPHABET: [char; 37] = [
    '_', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
    'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

impl Dummy<UniqueEmail> for String {
    fn dummy_with_rng<R: Rng + ?Sized>(_v: &UniqueEmail, _rng: &mut R) -> String {
        let email: String = SafeEmail().fake();
        let id = nanoid!(8, &ALPHABET);
        let i = email.find('@').unwrap();
        format!("{}.{}@{}", &email[..i], id, &email[i + 1..])
    }
}

async fn bulk_insert(users: HashSet<UserStat>, pool: &PgPool) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;
    for user in users {
        let query = sqlx::query("INSERT INTO user_stats VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)")
            .bind(&user.email)
            .bind(&user.name)
            .bind(&user.gender)
            .bind(user.created_at)
            .bind(user.last_visited_at)
            .bind(user.last_watched_at)
            .bind(&user.recent_watched)
            .bind(&user.viewed_but_not_started)
            .bind(&user.started_but_not_finished)
            .bind(&user.finished)
            .bind(user.last_email_notification)
            .bind(user.last_in_app_notification)
            .bind(user.last_sms_notification)
            ;
        tx.execute(query).await?;
    }
    tx.commit().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let pool = PgPool::connect("postgresql://postgres:postgres@localhost:5432/stats")
        .await
        .unwrap();
    for i in 1..=500 {
        let users: HashSet<_> = (0..10000).map(|_| Faker.fake::<UserStat>()).collect();
        let start = tokio::time::Instant::now();
        bulk_insert(users, &pool).await.unwrap();
        let end = start.elapsed().as_millis();
        println!("batch {} done in {}ms", i, end);
    }
}
