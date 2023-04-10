use ai_prompt::api::{AiEnv, CheckResult, AsEmbeddingEnv, AsEmbeddable, Embedding};
use rl_queue::{QuotaState, RateLimiter};
use std::{borrow::Borrow, time::Instant};
use tracing::{info, warn};

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
    pub async fn run<R, I: AsEmbeddable>(&mut self, items: &[I]) -> anyhow::Result<Vec<Embedding>> {
        self.token_rate_limiter.block_until_ok().await;
        self.api_calls_rate_limiter.block_until_ok().await;
        self.api_calls_rate_limiter.add(1 as f64);

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

        Ok(response.data)
    }
}
