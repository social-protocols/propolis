//! Database access via sqlx

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::env;
use std::str::FromStr;

use crate::structs::{StatementStats, User, Vote};
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
    pub async fn delete_content(&self, pool: &SqlitePool) -> Result<(), Error> {
        for table in vec!["authors", "votes", "vote_history", "queue"] {
            sqlx::query(format!("DELETE FROM {} WHERE user_id=?", table).as_str())
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
        let vote_i32 = vote as i32;
        sqlx::query!(
            "INSERT INTO votes (statement_id, user_id, vote)
            VALUES (?, ?, ?)
            on CONFLICT (statement_id, user_id)
            do UPDATE SET vote = excluded.vote",
            statement_id,
            self.id,
            vote_i32
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "INSERT INTO vote_history (user_id, statement_id, vote) VALUES (?, ?, ?)",
            self.id,
            statement_id,
            vote_i32
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

        match vote {
            Vote::Yes | Vote::No | Vote::ItDepends => {
                // add followups to queue
                sqlx::query!(
                    "insert into queue (user_id, statement_id) select ?, followup_id from followups where statement_id = ? on conflict do nothing",
                    self.id,
                    statement_id
                )
                .execute(pool)
                .await?;
            }
            Vote::Skip => {}
        };

        // update statement stats
        sqlx::query!(
            "
            insert or replace into statement_stats (statement_id, yes_votes, no_votes, skip_votes, itdepends_votes)
              select
            statement_id,
              coalesce(sum(vote == 1), 0) as yes_votes,
              coalesce(sum(vote == -1), 0) as no_votes,
              coalesce(sum(vote == 0), 0) as skip_votes,
              coalesce(sum(vote == 2), 0) as itdepends_votes
              from votes
              where statement_id = ?
              group by statement_id",
            statement_id
        ) 
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn is_following(&self, statement_id: i64, pool: &SqlitePool) -> Result<bool, Error> {
        let subscription = sqlx::query!(
            "select 1 as subscription from subscriptions where user_id = ? and statement_id = ?",
            self.id,
            statement_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(subscription.is_some())
    }

    pub async fn follow(&self, statement_id: i64, pool: &SqlitePool) -> Result<(), Error> {
        // insert into subscriptions
        sqlx::query!(
            "insert into subscriptions (user_id, statement_id) values (?, ?) on conflict do nothing",
            self.id,
            statement_id
        ).execute(pool).await?;

        // update statement stats
        sqlx::query!(
            "insert or replace into statement_stats (statement_id, subscriptions)
            values (?, 1) on conflict (statement_id) do update set subscriptions = statement_stats.subscriptions + 1",
            statement_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Returns all votes taken by a [User]
    pub async fn vote_history(&self, pool: &SqlitePool) -> Result<Vec<VoteHistoryItem>, Error> {
        Ok(sqlx::query_as!(
            VoteHistoryItem,
            "select s.id as statement_id, s.text as statement_text, timestamp as vote_timestamp, vote from vote_history v
            join statements s on s.id = v.statement_id
            where user_id = ? and vote != 0
            order by timestamp desc",
            self.id
            )
            .fetch_all(pool).await?)
    }

    pub async fn add_statement(
        &self,
        text: String,
        target_statement_id: Option<i64>,
        pool: &SqlitePool,
    ) -> Result<(), Error> {
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

        if let Some(target_statement_id) = target_statement_id {
            // add created statement as followup to target statement
            add_followup(target_statement_id, created_statement.id, pool).await?;

            // add to queue of subscribers of target
            sqlx::query!(
                "insert into queue (user_id, statement_id) select user_id, ? from subscriptions where statement_id = ? on conflict do nothing",
                created_statement.id,
                target_statement_id,
            ).execute(pool).await?;
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
            "select statement_id from queue where user_id = ? order by timestamp asc limit 1",
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
        .await?
    )
}

/// Create db connection & configure it
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
    sqlx::migrate!("./migrations")
        .run(&sqlite_pool)
        .await
        .expect("Unable to migrate");

    for option in vec![
        "pragma temp_store = memory;",
        "pragma mmap_size = 30000000000;",
        "pragma page_size = 4096;",
    ] {
        sqlx::query(option)
            .execute(&sqlite_pool)
            .await
            .expect(format!("Unable to set option: {}", option).as_str());
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

pub async fn get_statement(
    statement_id: i64,
    pool: &SqlitePool,
) -> Result<Option<Statement>, Error> {
    Ok(sqlx::query_as!(
        Statement,
        "SELECT id, text from statements where id = ?",
        statement_id,
    )
    .fetch_optional(pool)
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

#[derive(sqlx::FromRow)]
pub struct SubmissionsItem {
    pub statement_id: i64,
    pub statement_text: String,
    pub author_timestamp: i64,
    pub vote: i64, // vote is nullable, should be Option<i64>, but TODO: https://github.com/djc/askama/issues/752
    pub yes_count: i64,
    pub no_count: i64,
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

pub async fn add_followup(
    statement_id: i64,
    followup_id: i64,
    pool: &SqlitePool,
) -> Result<(), Error> {
    sqlx::query!(
        "INSERT INTO followups (statement_id, followup_id) VALUES (?, ?)",
        statement_id,
        followup_id,
    )
    .execute(pool)
    .await?;

    // add followup to queue of users who voted on the original statement
    sqlx::query!(
        "insert into queue (user_id, statement_id) select user_id, ? from votes where statement_id = ? and vote = 1 or vote = -1 on conflict do nothing",
        followup_id,
        statement_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
