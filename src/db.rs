use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::auth::User;

#[async_trait]
pub trait UserQueries {
    async fn num_statements(&self, _: &SqlitePool) -> i32 {
        0
    }
    async fn num_votes(&self, _: &SqlitePool) -> i32 {
        0
    }
}

#[async_trait]
impl UserQueries for User {

    async fn num_statements(&self, pool: &SqlitePool) -> i32 {
        sqlx::query!(
            "SELECT COUNT(*) as count FROM authors where user_id = ?",
            self.id,
        )
        .fetch_one(pool)
        .await
        .expect("Must be valid")
        .count
    }

    async fn num_votes(&self, pool: &SqlitePool) -> i32 {
        sqlx::query!(
            "SELECT COUNT(*) as count FROM votes where user_id = ?",
            self.id,
        )
        .fetch_one(pool)
        .await
        .expect("Must be valid")
        .count
    }
}
