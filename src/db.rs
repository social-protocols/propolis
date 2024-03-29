//! Database access via sqlx

use anyhow::Result;
use sqlx::SqlitePool;

use crate::structs::{StatementStats, TargetSegment, User, Vote};
#[cfg(feature = "with_predictions")]
use std::collections::HashMap;

use crate::{
    highlight::{HIGHLIGHT_BEGIN, HIGHLIGHT_END},
    structs::{SearchResultStatement, Statement, VoteHistoryItem},
};

#[cfg(feature = "with_predictions")]
pub struct UserStat {
    /// Total votes cast for this ideology / bfp trait. Disagree votes count towards negative
    pub votes_cast: i64,
    /// A normalized weight across all votes and all ideologies for this user
    pub votes_weight: f64,
}

impl User {
    /// Returns number of statements added by [User]
    pub async fn num_statements(&self, pool: &SqlitePool) -> Result<i32> {
        Ok(sqlx::query!(
            "SELECT COUNT(*) as count FROM authors where user_id = ?",
            self.id,
        )
        .fetch_one(pool)
        .await?
        .count)
    }

    /// Returns number of votes added by [User]
    pub async fn num_votes(&self, pool: &SqlitePool) -> Result<i32> {
        Ok(sqlx::query!(
            "SELECT COUNT(*) as count FROM votes where user_id = ?",
            self.id,
        )
        .fetch_one(pool)
        .await?
        .count)
    }

    /// Moves content from one user to another
    pub async fn move_content_to(&self, new_user: &User, pool: &SqlitePool) -> Result<()> {
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
    pub async fn delete_content(&self, pool: &SqlitePool) -> Result<()> {
        for table in ["authors", "votes", "vote_history", "queue"] {
            sqlx::query(format!("DELETE FROM {table} WHERE user_id=?").as_str())
                .bind(self.id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    /// Deletes user without content
    pub async fn delete(&self, pool: &SqlitePool) -> Result<()> {
        sqlx::query!("DELETE FROM users WHERE id=?", self.id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Votes on a statement
    pub async fn vote(&self, statement_id: i64, vote: Vote, pool: &SqlitePool) -> Result<()> {
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

    pub async fn is_subscribed(&self, statement_id: i64, pool: &SqlitePool) -> Result<bool> {
        let subscription = sqlx::query!(
            "select count(*) as subscription from subscriptions where user_id = ? and statement_id = ?",
            self.id,
            statement_id
        )
        .fetch_one(pool)
        .await?;
        Ok(subscription.subscription == 1)
    }

    pub async fn get_vote(&self, statement_id: i64, pool: &SqlitePool) -> Result<Option<Vote>> {
        let vote = sqlx::query_scalar!(
            "select vote from votes where user_id = ? and statement_id = ?",
            self.id,
            statement_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(vote.and_then(|v| Vote::from(v).ok()))
    }

    pub async fn subscribe(&self, statement_id: i64, pool: &SqlitePool) -> Result<()> {
        sqlx::query!(
            "insert into subscriptions (user_id, statement_id) values (?, ?) on conflict do nothing",
            self.id,
            statement_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Returns a HashMap of BFP Trait ⇒ Votecount for user
    #[cfg(feature = "with_predictions")]
    pub async fn bfp_traits_votes(&self, pool: &SqlitePool) -> Result<HashMap<String, i64>> {
        use crate::prediction::prompts::Score;
        let vhist = self.vote_history(1000, pool).await?;
        let mut votes: HashMap<String, i64> = HashMap::new();
        for VoteHistoryItem {
            statement_id,
            statement_text,
            vote_timestamp: _,
            vote,
        } in vhist
        {
            let stmt = Statement {
                id: statement_id,
                text: statement_text,
            };
            if let Some(crate::prediction::prompts::StatementMeta::Personal {
                tags: _,
                bfp_traits,
            }) = stmt.get_meta(pool).await?
            {
                for bfpt_val in bfp_traits {
                    let score: i64 = match bfpt_val.score {
                        Score::Strong => 1,
                        _ => continue,
                    };
                    let factor = Vote::from(vote)?.to_factor();
                    votes
                        .entry(bfpt_val.value)
                        .and_modify(|i| *i += score * factor)
                        .or_insert(1);
                }
            }
        }
        Ok(votes)
    }

    /// Returns a HashMap of Ideology ⇒ Votecount for user
    #[cfg(feature = "with_predictions")]
    pub async fn ideology_votes(&self, pool: &SqlitePool) -> Result<HashMap<String, i64>> {
        use crate::prediction::prompts::Score;
        let vhist = self.vote_history(1000, pool).await?;
        let mut votes: HashMap<String, i64> = HashMap::new();
        for VoteHistoryItem {
            statement_id,
            statement_text,
            vote_timestamp: _,
            vote,
        } in vhist
        {
            let stmt = Statement {
                id: statement_id,
                text: statement_text,
            };
            if let Some(crate::prediction::prompts::StatementMeta::Politics {
                tags: _,
                ideologies,
            }) = stmt.get_meta(pool).await?
            {
                for ideology_val in ideologies {
                    let score: i64 = match ideology_val.score {
                        Score::Strong => 1,
                        _ => continue,
                    };
                    let factor = Vote::from(vote)?.to_factor();
                    votes
                        .entry(ideology_val.value)
                        .and_modify(|i| *i += score * factor)
                        .or_insert(1);
                }
            }
        }
        Ok(votes)
    }

    #[cfg(feature = "with_predictions")]
    pub async fn stats_map(
        &self,
        votes: &HashMap<String, i64>,
    ) -> Result<HashMap<String, UserStat>> {
        let mut sorted_vec: Vec<(&String, &i64)> = votes.iter().collect();
        sorted_vec.sort_by(|a, b| b.1.cmp(a.1));
        let largest_value = sorted_vec.first().map(|i| *i.1).unwrap_or(0);
        // we determine the smallest value to offset all values upwards so lowest starts at 0
        // FIXME:
        // - Map values in [0, inf] to [0, 1] *and*
        // - Map values from [-inf, 0] to [-1, 0]
        // - This way, unvoted entries should be at 0 instead of being pulled towards the side with
        //   a larger amplitude
        let smallest_value = sorted_vec.last().map(|i| *i.1).unwrap_or(0);

        let weighted_hmap: HashMap<String, UserStat> = votes
            .iter()
            .map(|(k, v)| {
                (
                    k.to_owned(),
                    UserStat {
                        votes_cast: *v,
                        votes_weight: (*v - smallest_value) as f64
                            / ((largest_value - smallest_value) as f64).max(1.0),
                    },
                )
            })
            .collect();
        Ok(weighted_hmap)
    }

    /// Return a hashmap with weighted ideologies
    #[cfg(feature = "with_predictions")]
    pub async fn ideology_stats_map(&self, pool: &SqlitePool) -> Result<HashMap<String, UserStat>> {
        let votes = self.ideology_votes(pool).await?;

        self.stats_map(&votes).await
    }

    /// Return a hashmap with weighted bfp traits
    #[cfg(feature = "with_predictions")]
    pub async fn bfp_traits_map(&self, pool: &SqlitePool) -> Result<HashMap<String, UserStat>> {
        let votes = self.bfp_traits_votes(pool).await?;

        self.stats_map(&votes).await
    }

    /// Returns all votes taken by a [User]
    // TODO: just return what's in the vote_history table
    pub async fn vote_history(
        &self,
        limit: i32,
        pool: &SqlitePool,
    ) -> Result<Vec<VoteHistoryItem>> {
        Ok(sqlx::query_as!(
            VoteHistoryItem,
            "select s.id as statement_id, s.text as statement_text, v.created as vote_timestamp, vote from vote_history v
            join statements s on s.id = v.statement_id
            where user_id = ? and vote != 0
            order by v.created desc
            limit ?
            ",
            self.id,
            limit
            )
            .fetch_all(pool).await?)
    }

    pub async fn add_statement(&self, text: &str, pool: &SqlitePool) -> Result<i64> {
        // TODO: add statement and author entry in transaction
        // TODO: no compile time check here, because of foreign-key bug in sqlx: https://github.com/launchbadge/sqlx/issues/2449
        let created_statement_id =
            sqlx::query_scalar::<_, i64>("INSERT INTO statements (text) VALUES (?) RETURNING id")
                .bind(text)
                .fetch_one(pool)
                .await?;

        sqlx::query!(
            "INSERT INTO authors (user_id, statement_id) VALUES (?, ?)",
            self.id,
            created_statement_id
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "INSERT INTO subscriptions (user_id, statement_id) VALUES (?, ?)",
            self.id,
            created_statement_id
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "INSERT INTO queue (user_id, statement_id) VALUES (?, ?)",
            self.id,
            created_statement_id
        )
        .execute(pool)
        .await?;

        Ok(created_statement_id)
    }

    // Retrieve next statement id for [User]
    pub async fn next_statement_for_user(&self, pool: &SqlitePool) -> Result<Option<i64>> {
        // try to pick a statement from the user's personal queue
        let statement_id = self.next_statement_id_from_queue(pool).await?;

        // if there is no statement in the queue, pick a random statement
        Ok(match statement_id {
            Some(statement_id) => Some(statement_id),
            None => self.random_unvoted_statement_id(pool).await?,
        })
    }

    /// Retrieve next statement id from [User] queue
    pub async fn next_statement_id_from_queue(&self, pool: &SqlitePool) -> Result<Option<i64>> {
        // TODO: sqlx bug: adding `order by timestamp` infers wrong type in macro
        Ok(sqlx::query_scalar::<_, i64>(
            "select statement_id from queue where user_id = ? order by created asc limit 1",
        )
        .bind(self.id)
        .fetch_optional(pool)
        .await?)
    }

    pub async fn random_unvoted_statement_id(&self, pool: &SqlitePool) -> Result<Option<i64>> {
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
  created as \"created!\"
from statement_predictions
where statement_id = ? order by created desc",
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

pub async fn statement_stats(statement_id: i64, pool: &SqlitePool) -> Result<StatementStats> {
    Ok(
        // TODO: sqlx bug: computed column types are wrong
        sqlx::query_as::<_, StatementStats>(
            "SELECT
            yes_votes, no_votes, skip_votes, subscriptions, cast(total_votes as int) as total_votes, participation, polarization, votes_per_subscription
            FROM statement_stats where statement_id = ?")
        .bind(statement_id)
        .fetch_one(pool)
        .await.unwrap_or_else(|_| StatementStats::empty()),
    )
}

pub async fn top_statements(pool: &SqlitePool) -> Result<Vec<Statement>> {
    Ok(sqlx::query_as::<_,Statement>("select stats.statement_id as id, s.text as text from statement_stats stats join statements s on s.id = stats.statement_id order by polarization desc, polarization asc limit 10").fetch_all(pool).await?)
}

pub async fn random_statement_id(pool: &SqlitePool) -> Result<Option<i64>> {
    // for anonymous users, pick a random statement
    Ok(sqlx::query_scalar::<_, i64>(
        // TODO: https://github.com/launchbadge/sqlx/issues/1524
        "SELECT id from statements ORDER BY RANDOM() LIMIT 1",
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn get_statement(statement_id: i64, pool: &SqlitePool) -> Result<Statement> {
    Ok(sqlx::query_as!(
        Statement,
        "SELECT id, text from statements where id = ?",
        statement_id,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn search_statement(text: &str, pool: &SqlitePool) -> Result<Vec<SearchResultStatement>> {
    if text.is_empty() {
        return Ok(vec![]);
    }

    Ok(sqlx::query_as::<_, SearchResultStatement>(
        "SELECT id, text as text_original, highlight(statements_fts, 1, ?, ?) as text_highlighted
        FROM statements_fts
        WHERE text MATCH ?
        LIMIT 25",
    )
    .bind(HIGHLIGHT_BEGIN)
    .bind(HIGHLIGHT_END)
    .bind(text)
    .fetch_all(pool)
    .await?)
}

pub async fn get_subscriptions(user: &User, pool: &SqlitePool) -> Result<Vec<Statement>> {
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
) -> Result<()> {
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

pub async fn get_followups(statement_id: i64, pool: &SqlitePool) -> Result<Vec<i64>> {
    Ok(sqlx::query_scalar!(
        "select followup_id from followups where statement_id = ?",
        statement_id,
    )
    .fetch_all(pool)
    .await?)
}
