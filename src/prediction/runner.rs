use std::{collections::HashMap, time::{Duration, Instant}};

use rl_queue::{RateLimiter, QuotaState};

use crate::structs::Statement;

/// Returns those statements that are not yet predicted
pub async fn unpredicted_statements(_num: u32) -> anyhow::Result<Vec<Statement>> {
    Ok(vec![])
}

pub struct StatementCategorizationPredictor {
    /// Used to set a rate based on the amount of tokens that we have used overall
    token_rate_limiter: RateLimiter,
}

impl StatementCategorizationPredictor {
    pub async fn predict_next_statement(&mut self) -> anyhow::Result<()> {
        if self.token_rate_limiter.check() {
            for s in unpredicted_statements(1).await? {
                // TODO: run openai
            }
        }
        Ok(())
    }
}

pub async fn run() {
    let mut pred = StatementCategorizationPredictor {
        token_rate_limiter: RateLimiter::new(100.0, Duration::from_secs(1))
    };
    loop {
        if !pred.token_rate_limiter.check() {
            continue;
        }
        println!("Adding 50");
        match pred.token_rate_limiter.add(51) {
            QuotaState::ExceededUntil(instant) => {
                async_std::task::sleep( instant - Instant::now() ).await;
            }
            QuotaState::Remaining(v) => {
                println!("Remaining: {}", v);
            }
        }
    }
}
