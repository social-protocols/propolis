use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use rl_queue::{QuotaState, RateLimiter};
use sqlx::SqlitePool;
use tracing::log::{info, warn};

use crate::{prediction::{prompts::multi_statement_predictor, prediction}, structs::Statement};

use super::openai::{OpenAiEnv, OpenAiModel};

/// Returns those statements that are not yet predicted
pub async fn unpredicted_statements(num: u32, pool: &SqlitePool) -> anyhow::Result<Vec<Statement>> {
    Ok(sqlx::query_as!(
        Statement,
        "SELECT * FROM statements WHERE
id NOT IN (SELECT statement_id FROM statement_predictions)
LIMIT ?",
        num
    )
    .fetch_all(pool)
    .await
    .expect("Unable to fetch"))
}

pub struct StatementCategorizationPredictor {
    /// Used to set a rate based on the amount of tokens that we have used overall
    token_rate_limiter: RateLimiter,
}

impl StatementCategorizationPredictor {
    pub async fn predict_next_statement(&mut self, pool: &SqlitePool) -> anyhow::Result<()> {
        let env = OpenAiEnv::from(OpenAiModel::Gpt35Turbo);
        if self.token_rate_limiter.check() {
            // TODO: predict multiple statements
            let stmts = unpredicted_statements(1, &pool).await?;
            for statement in &stmts {
                info!("Predicting statement ({}): {}", statement.id, statement.text);
                let pred = prediction::run(
                    &statement,
                    multi_statement_predictor(stmts.as_slice()),
                    &env,
                    &pool,
                )
                .await?;
                match self.token_rate_limiter.add(pred.total_tokens as f64) {
                    QuotaState::ExceededUntil(exceeded_by, instant) => {
                        warn!("Exceeded token quota by {}. Waiting", exceeded_by);
                        async_std::task::sleep( instant - Instant::now() ).await;
                    }
                    QuotaState::Remaining(v) => {

                        info!("Quota remaining: {}", v);
                    }
                }
            }
        }
        Ok(())
    }
}

pub async fn run(pool: &SqlitePool) {
    let mut pred = StatementCategorizationPredictor {
        token_rate_limiter: RateLimiter::new(100.0, Duration::from_secs(30)),
    };
    loop {
        pred.predict_next_statement(&pool).await.unwrap();
    }
}
