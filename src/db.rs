use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::auth::User;

#[async_trait]
pub trait UserQueries {
    async fn num_statements(&self, _: &SqlitePool) -> i32 {
        0
    }

    async fn num_votes(&self, _: &SqlitePool) -> i32 {
        0
    }

    async fn move_content_to(&self, _new_user: &User, _pool: &SqlitePool) {}
    async fn delete_content(&self, _pool: &SqlitePool) {}

    async fn delete(&self, _: &SqlitePool) {}
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

    /// Moves content from one user to another
    async fn move_content_to(&self, new_user: &User, pool: &SqlitePool) {
        for table in vec!["authors", "votes", "vote_history", "queue"] {
            sqlx::query(format!("UPDATE {} SET user_id=? WHERE user_id=?", table).as_str())
                .bind(new_user.id)
                .bind(self.id)
                .execute(pool)
                .await
                .expect("Update should work");
        }
    }

    /// Deletes all content of a particular user
    async fn delete_content(&self, pool: &SqlitePool) {
        for table in vec!["authors", "votes", "vote_history", "queue"] {
            sqlx::query(format!("DELETE FROM {} WHERE user_id=?", table).as_str())
                .bind(self.id)
                .execute(pool)
                .await
                .expect("Delete should work");
        }
    }

    /// Deletes user without content
    async fn delete(&self, pool: &SqlitePool) {
        sqlx::query!("DELETE FROM users WHERE id=?", self.id)
            .execute(pool)
            .await
            .expect("Delete should work");
    }
}
