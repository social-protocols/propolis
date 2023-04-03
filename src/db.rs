//! Database access via sqlx

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::env;
use std::str::FromStr;

use crate::structs::{StatementStats, TargetSegment, User, Vote};
use crate::{
    error::Error,
    structs::{Statement, VoteHistoryItem},
};

impl User {
    /// Returns number of statements added by [User]
    pub async fn num_statements(&self, pool: &SqlitePool) -> Result<i32, Error> {
        Ok(sqlx::query!(
            "SELECT COUNT(*) as count FROM authors where user_id = ?",
            self.id,
        )
        .fetch_one(pool)
        .await?
        .count)
    }

    /// Returns number of votes added by [User]
    pub async fn num_votes(&self, pool: &SqlitePool) -> Result<i32, Error> {
        Ok(sqlx::query!(
            "SELECT COUNT(*) as count FROM votes where user_id = ?",
            self.id,
        )
        .fetch_one(pool)
        .await?
        .count)
    }

    /// Moves content from one user to another
    pub async fn move_content_to(&self, new_user: &User, pool: &SqlitePool) -> Result<(), Error> {
        for table in ["authors", "votes", "vote_history", "queue"] {
            sqlx::query(format!("UPDATE {table} SET user_id=? WHERE user_id=?").as_str())
                .bind(new_user.id)
                .bind(self.id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    /// Deletes all content of a particular user
    pub async fn delete_content(&self, pool: &SqlitePool) -> Result<(), Error> {
        for table in ["authors", "votes", "vote_history", "queue"] {
            sqlx::query(format!("DELETE FROM {table} WHERE user_id=?").as_str())
                .bind(self.id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    /// Deletes user without content
    pub async fn delete(&self, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query!("DELETE FROM users WHERE id=?", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Votes on a statement
    pub async fn vote(
        &self,
        statement_id: i64,
        vote: Vote,
        pool: &SqlitePool,
    ) -> Result<(), Error> {
        // TODO: voting on specialization gets generalizations and other specializations into queue

        let vote_i32 = vote as i32;
        sqlx::query!(
            "INSERT INTO vote_history (user_id, statement_id, vote) VALUES (?, ?, ?)",
            self.id,
            statement_id,
            vote_i32
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn is_subscribed(&self, statement_id: i64, pool: &SqlitePool) -> Result<bool, Error> {
        let subscription = sqlx::query!(
            "select count(*) as subscription from subscriptions where user_id = ? and statement_id = ?",
            self.id,
            statement_id
        )
        .fetch_one(pool)
        .await?;
        Ok(subscription.subscription == 1)
    }

    pub async fn get_vote(
        &self,
        statement_id: i64,
        pool: &SqlitePool,
    ) -> Result<Option<Vote>, Error> {
        let vote = sqlx::query_scalar!(
            "select vote from votes where user_id = ? and statement_id = ?",
            self.id,
            statement_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(vote.map(|v| Vote::from(v).unwrap()))
    }

    pub async fn subscribe(&self, statement_id: i64, pool: &SqlitePool) -> Result<(), Error> {
        sqlx::query!(
            "insert into subscriptions (user_id, statement_id) values (?, ?) on conflict do nothing",
            self.id,
            statement_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Returns all votes taken by a [User]
    // TODO: just return what's in the vote_history table
    pub async fn vote_history(&self, pool: &SqlitePool) -> Result<Vec<VoteHistoryItem>, Error> {
        Ok(sqlx::query_as!(
            VoteHistoryItem,
            "select s.id as statement_id, s.text as statement_text, v.created as vote_timestamp, vote from vote_history v
            join statements s on s.id = v.statement_id
            where user_id = ? and vote != 0
            order by v.created desc",
            self.id
            )
            .fetch_all(pool).await?)
    }

    pub async fn add_statement(
        &self,
        text: String,
        target_segment: Option<TargetSegment>,
        pool: &SqlitePool,
    ) -> Result<(), Error> {
        // TODO: track specialization for it-depends creations
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
            "INSERT INTO subscriptions (user_id, statement_id) VALUES (?, ?)",
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

        if let Some(target_segment) = target_segment {
            // add created statement as followup to target statement
            add_followup(target_segment, created_statement.id, pool).await?;
        }

        Ok(())
    }

    // Retrieve next statement id for [User]
    pub async fn next_statement_for_user(&self, pool: &SqlitePool) -> Result<Option<i64>, Error> {
        // try to pick a statement from the user's personal queue
        let statement_id = self.next_statement_id_from_queue(pool).await?;

        // if there is no statement in the queue, pick a random statement
        Ok(match statement_id {
            Some(statement_id) => Some(statement_id),
            None => self.random_unvoted_statement_id(pool).await?,
        })
    }

    /// Retrieve next statement id from [User] queue
    pub async fn next_statement_id_from_queue(
        &self,
        pool: &SqlitePool,
    ) -> Result<Option<i64>, Error> {
        // TODO: sqlx bug: adding `order by timestamp` infers wrong type in macro
        Ok(sqlx::query_scalar::<_, i64>(
            "select statement_id from queue where user_id = ? order by created asc limit 1",
        )
        .bind(self.id)
        .fetch_optional(pool)
        .await?)
    }

    pub async fn random_unvoted_statement_id(
        &self,
        pool: &SqlitePool,
    ) -> Result<Option<i64>, Error> {
        Ok(sqlx::query_scalar::<_, i64>(
            "select id from statements where id not in (select statement_id from votes v where v.user_id = ?) order by random() limit 1")
            .bind(self.id)
            .fetch_optional(pool)
            .await?)
    }
}

impl Statement {
    #[cfg(feature = "with_predictions")]
    pub async fn get_meta(
        &self,
        pool: &SqlitePool,
    ) -> anyhow::Result<Option<crate::prediction::prompts::StatementMeta>> {
        use crate::structs::StatementPrediction;

        let pred = sqlx::query_as!(
            StatementPrediction,
            "select
-- see: https://github.com/launchbadge/sqlx/issues/1126 on why this is necessary when using ORDER BY
  statement_id as \"statement_id!\",
  ai_env as \"ai_env!\",
  prompt_name as \"prompt_name!\",
  prompt_version as \"prompt_version!\",
  prompt_result as \"prompt_result!\",
  completion_tokens as \"completion_tokens!\",
  prompt_tokens as \"prompt_tokens!\",
  total_tokens as \"total_tokens!\",
  timestamp as \"timestamp!\"
from statement_predictions
where statement_id = ? order by timestamp desc",
            self.id
        )
        .fetch_optional(pool)
        .await?;

        match pred {
            Some(pred) => Ok(Some(serde_json::from_str::<
                crate::prediction::prompts::StatementMeta,
            >(pred.prompt_result.as_str())?)),
            None => Ok(None),
        }
    }
}

pub async fn statement_stats(
    statement_id: i64,
    pool: &SqlitePool,
) -> Result<StatementStats, Error> {
    Ok(
        // TODO: sqlx bug: computed column types are wrong
        sqlx::query_as::<_, StatementStats>(
            "SELECT
            yes_votes, no_votes, skip_votes, itdepends_votes, subscriptions, cast(total_votes as int) as total_votes, participation, polarization, votes_per_subscription
            FROM statement_stats where statement_id = ?")
        .bind(statement_id)
        .fetch_one(pool)
        .await.unwrap_or_else(|_| StatementStats::empty()),
    )
}

/// Create db connection & configure it
//TODO: move to own file
pub async fn setup_db() -> SqlitePool {
    // high performance sqlite insert example: https://kerkour.com/high-performance-rust-with-sqlite
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // if embed_migrations is enabled, we create the database if it doesn't exist
    let create_database_if_missing = cfg!(feature = "embed_migrations");

    let connection_options = SqliteConnectOptions::from_str(&database_url)
        .unwrap()
        .create_if_missing(create_database_if_missing)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(30));

    let sqlite_pool = SqlitePoolOptions::new()
        .max_connections(8)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect_with(connection_options)
        .await
        .unwrap();

    #[cfg(feature = "embed_migrations")]
    {
        println!("Running database migrations...");
        sqlx::migrate!("./migrations")
            .run(&sqlite_pool)
            .await
            .expect("Unable to migrate");
    }

    for option in [
        "pragma temp_store = memory;",
        "pragma mmap_size = 30000000000;",
        "pragma page_size = 4096;",
    ] {
        sqlx::query(option)
            .execute(&sqlite_pool)
            .await
            .unwrap_or_else(|_| panic!("Unable to set option: {option}"));
    }

    sqlite_pool
}

pub async fn random_statement_id(pool: &SqlitePool) -> Result<Option<i64>, Error> {
    // for anonymous users, pick a random statement
    Ok(sqlx::query_scalar::<_, i64>(
        // TODO: https://github.com/launchbadge/sqlx/issues/1524
        "SELECT id from statements ORDER BY RANDOM() LIMIT 1",
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn get_statement(statement_id: i64, pool: &SqlitePool) -> Result<Statement, Error> {
    Ok(sqlx::query_as!(
        Statement,
        "SELECT id, text from statements where id = ?",
        statement_id,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn autocomplete_statement(
    text: &str,
    pool: &SqlitePool,
) -> Result<Vec<Statement>, Error> {
    Ok(sqlx::query_as::<_, Statement>(
        "SELECT id, highlight(statements_fts, 1, '<b>', '</b>') as text
FROM statements_fts
WHERE text MATCH ?
LIMIT 25",
    )
    .bind(text)
    .fetch_all(pool)
    .await?)
}

pub async fn get_subscriptions(user: &User, pool: &SqlitePool) -> Result<Vec<Statement>, Error> {
    // TODO: https://github.com/launchbadge/sqlx/issues/1524
    Ok(sqlx::query_as::<_, Statement>(
        "select s.id, s.text from subscriptions sub join statements s on s.id = sub.statement_id where sub.user_id = ?",
    )
    .bind(user.id)
    .fetch_all(pool)
    .await?)
}

pub async fn add_followup(
    segment: TargetSegment,
    followup_id: i64,
    pool: &SqlitePool,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO followups (statement_id, followup_id, target_yes, target_no) VALUES (?, ?, ?, ?)
         on conflict(statement_id, followup_id) do update
         set target_yes = min(1, target_yes + excluded.target_yes),
             target_no  = min(1, target_no  + excluded.target_no )",
        segment.statement_id,
        followup_id,
        segment.voted_yes,
        segment.voted_no
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_followups(statement_id: i64, pool: &SqlitePool) -> Result<Vec<i64>, Error> {
    Ok(sqlx::query_scalar!(
        "select followup_id from followups where statement_id = ?",
        statement_id,
    )
    .fetch_all(pool)
    .await?)
}
