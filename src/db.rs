use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::{auth::User, error::Error, pages::submissions::SubmissionsItem, structs::Statement};

#[derive(sqlx::FromRow)]
pub struct VoteHistoryItem {
    pub statement_id: i64,
    pub statement_text: String,
    pub vote_timestamp: i64,
    pub vote: i64,
}

#[async_trait]
pub trait UserQueries {
    async fn num_statements(&self, _: &SqlitePool) -> Result<i32, Error>;
    async fn num_votes(&self, _: &SqlitePool) -> Result<i32, Error>;

    async fn move_content_to(&self, _new_user: &User, _pool: &SqlitePool) -> Result<(), Error>;
    async fn delete_content(&self, _pool: &SqlitePool) -> Result<(), Error>;

    async fn delete(&self, _: &SqlitePool) -> Result<(), Error>;

    /// Casts a vote
    async fn vote(&self, statement_id: i64, vote: i32, pool: &SqlitePool) -> Result<(), Error>;
    /// Returns vote history
    async fn vote_history(&self, pool: &SqlitePool) -> Result<Vec<VoteHistoryItem>, Error>;
    /// Adds a statement
    async fn add_statement(&self, text: String, pool: &SqlitePool) -> Result<(), Error>;
}

#[async_trait]
impl UserQueries for User {
    async fn num_statements(&self, pool: &SqlitePool) -> Result<i32, Error> {
        Ok(sqlx::query!(
            "SELECT COUNT(*) as count FROM authors where user_id = ?",
            self.id,
        )
        .fetch_one(pool)
        .await?
        .count)
    }

    async fn num_votes(&self, pool: &SqlitePool) -> Result<i32, Error> {
        Ok(sqlx::query!(
            "SELECT COUNT(*) as count FROM votes where user_id = ?",
            self.id,
        )
        .fetch_one(pool)
        .await?
        .count)
    }

    /// Moves content from one user to another
    async fn move_content_to(&self, new_user: &User, pool: &SqlitePool) -> Result<(), Error> {
        for table in vec!["authors", "votes", "vote_history", "queue"] {
            sqlx::query(format!("UPDATE {} SET user_id=? WHERE user_id=?", table).as_str())
                .bind(new_user.id)
                .bind(self.id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    /// Deletes all content of a particular user
    async fn delete_content(&self, pool: &SqlitePool) -> Result<(), Error> {
        for table in vec!["authors", "votes", "vote_history", "queue"] {
            sqlx::query(format!("DELETE FROM {} WHERE user_id=?", table).as_str())
                .bind(self.id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    /// Deletes user without content
    async fn delete(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query!("DELETE FROM users WHERE id=?", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Votes on a statement
    async fn vote(&self, statement_id: i64, vote: i32, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query!(
            "INSERT INTO votes (statement_id, user_id, vote)
VALUES (?, ?, ?)
on CONFLICT (statement_id, user_id)
do UPDATE SET vote = excluded.vote",
            statement_id,
            self.id,
            vote
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "INSERT INTO vote_history (user_id, statement_id, vote) VALUES (?, ?, ?)",
            self.id,
            statement_id,
            vote
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "delete from queue where user_id = ? and statement_id = ?",
            self.id,
            statement_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn vote_history(&self, pool: &SqlitePool) -> Result<Vec<VoteHistoryItem>, Error> {
        Ok(sqlx::query_as!(
            VoteHistoryItem,
            "
select s.id as statement_id, s.text as statement_text, timestamp as vote_timestamp, vote from vote_history v
join statements s on
  s.id = v.statement_id
where user_id = ? and vote != 0
order by timestamp desc", self.id)
            .fetch_all(pool).await?)
    }

    async fn add_statement(&self, text: String, pool: &SqlitePool) -> Result<(), Error> {
        // TODO: add statement and author entry in transaction
        let created_statement = sqlx::query!(
            "INSERT INTO statements (text) VALUES (?) RETURNING id",
            text
        )
        .fetch_one(pool)
        .await?;

        sqlx::query!(
            "INSERT INTO authors (user_id, statement_id) VALUES (?, ?)",
            self.id,
            created_statement.id
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "INSERT INTO queue (user_id, statement_id) VALUES (?, ?)",
            self.id,
            created_statement.id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

pub async fn get_statement(
    statement_id: i64,
    pool: &SqlitePool,
) -> Result<Option<Statement>, Error> {
    Ok(sqlx::query_as!(
        Statement,
        // TODO: https://github.com/launchbadge/sqlx/issues/1524
        "SELECT id, text from statements where id = ?",
        statement_id,
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn get_submissions(
    user: &User,
    pool: &SqlitePool,
) -> Result<Vec<SubmissionsItem>, Error> {
    // TODO: https://github.com/launchbadge/sqlx/issues/1524
    Ok(sqlx::query_as::<_, SubmissionsItem>(
        "
select
  s.id as statement_id,
  s.text as statement_text,
  a.timestamp as author_timestamp,
  v.vote as vote,
  coalesce(sum(v_stats.vote == 1), 0) as yes_count,
  coalesce(sum(v_stats.vote == -1), 0) as no_count
from authors a
join statements s on s.id = a.statement_id
left outer join votes v on
  s.id = v.statement_id and a.user_id = v.user_id
left outer join votes v_stats on
  v_stats.statement_id = a.statement_id
where a.user_id = ?
group by a.statement_id
order by a.timestamp desc",
    )
    .bind(user.id)
    .fetch_all(pool)
    .await?)
}
