use std::time::{Duration, Instant};

use rl_queue::{QuotaState, RateLimiter};
use sqlx::SqlitePool;
use tracing::log::{info, warn};

use crate::{prediction::prompts::MultiStatementPredictorV1, structs::Statement};

use super::{
    api::AiEnv,
    multi_statement_classifier::{
        MultiStatementPrompt, MultiStatementPromptGen, MultiStatementPromptResult,
        MultiStatementResultTypes, StatementMetaContainer,
    },
    openai::{OpenAiEnv, OpenAiModel},
};

/// Returns those statements that are not yet predicted
pub async fn unpredicted_statements(num: u32, pool: &SqlitePool) -> anyhow::Result<Vec<Statement>> {
    Ok(sqlx::query_as!(
        Statement,
        "SELECT id,text FROM statements WHERE
id NOT IN (SELECT statement_id FROM statement_predictions)
LIMIT ?",
        num
    )
    .fetch_all(pool)
    .await
    .expect("Unable to fetch"))
}

/// Runs given prompts and yields results
pub struct PromptRunner<'a, E: AiEnv + 'a> {
    /// Used to set a rate based on the amount of tokens that we have used overall
    token_rate_limiter: RateLimiter,
    env: &'a E,
}

impl<'a, E: AiEnv> PromptRunner<'a, E> {
    pub async fn predict_next_statement(&mut self, pool: &SqlitePool) -> anyhow::Result<()> {
        let env = OpenAiEnv::from(OpenAiModel::Gpt35Turbo);
        if self.token_rate_limiter.check() {
            // TODO: predict multiple statements
            let stmts = unpredicted_statements(1, &pool).await?;
            for statement in &stmts {
                info!(
                    "Predicting statement ({}): {}",
                    statement.id, statement.text
                );
                let stmts = vec![statement];
                let pred = MultiStatementPredictorV1 {};
                let result = pred.run(&stmts, &env, &pool).await?;
                let first = result.first().unwrap();
                match self.token_rate_limiter.add(first.total_tokens as f64) {
                    QuotaState::ExceededUntil(exceeded_by, instant) => {
                        warn!("Exceeded token quota by {}. Waiting", exceeded_by);
                        async_std::task::sleep(instant - Instant::now()).await;
                    }
                    QuotaState::Remaining(v) => {
                        info!("Quota remaining: {}", v);
                    }
                }
            }
        }
        Ok(())
    }

    /// Run the given prompt and return the result
    pub async fn run<R>(
        &mut self,
        prompt: MultiStatementPrompt<R>,
    ) -> anyhow::Result<MultiStatementPromptResult<R>>
    where
        R: MultiStatementResultTypes,
    {
        let result: Option<MultiStatementPromptResult<R>>;
        loop {
            if self.token_rate_limiter.check() {
                let response = self.env.send_prompt(&prompt).await?;
                match self
                    .token_rate_limiter
                    .add(response.response.total_tokens as f64)
                {
                    QuotaState::ExceededUntil(exceeded_by, _instant) => {
                        warn!("Exceeded token quota by {}. Waiting until quota reset", exceeded_by);
                    }
                    QuotaState::Remaining(v) => {
                        info!("Quota remaining: {}", v);
                    }
                }
                result = Some(response);
                break;
            } else {
                async_std::task::sleep(Duration::from_secs(1)).await;
            }
        }

        // unwrap fine, since the loops only breaks if there is a result
        Ok(result.unwrap())
    }
}

/// Setup continuous prompt generation and runner in an async loop
pub async fn run(pool: &SqlitePool) {
    let env = OpenAiEnv::from(OpenAiModel::Gpt35Turbo);

    let prompt_gen = MultiStatementPromptGen::<StatementMetaContainer> {
        batch_size: 5,
        prompt: |stmts| MultiStatementPrompt::statement_meta(&stmts),
        pool,
    };

    let mut runner = PromptRunner {
        token_rate_limiter: RateLimiter::new(100.0, Duration::from_secs(30)),
        env: &env,
    };
    loop {
        let prompt = prompt_gen.next_prompt().await.unwrap();
        match prompt {
            Some(prompt) => {
                info!("Running prompt: {}, V{}", prompt.name, prompt.version);
                let result = runner
                    .run(prompt)
                    .await
                    .unwrap();

                result.store(pool).await.unwrap();
            }
            None => {
                async_std::task::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
