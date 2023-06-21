use ai_prompt::api::{AsEmbeddable, AsEmbeddingEnv, Embedding};
use rl_queue::{QuotaState, RateLimiter};
use sqlx::SqlitePool;
use std::time::Instant;
use tracing::{info, warn};

use crate::structs::Statement;

/// Used to select statements from the db for various uses
pub struct StatementSelector {}

impl StatementSelector {
    /// Returns the next statements to embed
    pub async fn next_for_embedding(&self, pool: &SqlitePool) -> anyhow::Result<Vec<Statement>> {
        Ok(sqlx::query_as!(
            Statement,
            "
            SELECT id, text from statements
            WHERE id NOT IN (SELECT id FROM statement_embeddings)
            LIMIT 100"
        )
        .fetch_all(pool)
        .await?)
    }
}

/// Runner for calculating embeddings
pub struct EmbeddingsRunner<'a, E: AsEmbeddingEnv + 'a> {
    /// Used to set a rate based on the amount of tokens that we have used overall
    pub token_rate_limiter: RateLimiter,
    /// Used to set a rate based on how many API calls were done
    pub api_calls_rate_limiter: RateLimiter,
    pub env: &'a E,
}

impl<'a, E: AsEmbeddingEnv> EmbeddingsRunner<'a, E> {
    /// Run the given prompt and return the result
    pub async fn run<I: AsEmbeddable>(
        &mut self,
        items: &[I],
    ) -> anyhow::Result<(Vec<Embedding>, u32)> {
        self.token_rate_limiter.block_until_ok().await;
        self.api_calls_rate_limiter.block_until_ok().await;
        self.api_calls_rate_limiter.add(1_f64);

        let response = self
            .env
            .embed(
                items
                    .iter()
                    .map(|s| s.borrow())
                    .collect::<Vec<&str>>()
                    .as_slice(),
            )
            .await?;
        let ttokens = response.total_tokens;
        match self.token_rate_limiter.add(ttokens as f64) {
            QuotaState::ExceededUntil(exceeded_by, instant) => {
                warn!(
                    "Exceeded token quota by {}. Waiting for: {}s",
                    exceeded_by,
                    (instant - Instant::now()).as_secs()
                );
            }
            QuotaState::Remaining(v) => {
                info!("Quota remaining: {}", v);
            }
        }

        Ok((response.data, ttokens))
    }
}
