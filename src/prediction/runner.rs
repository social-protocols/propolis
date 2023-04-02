use std::time::{Duration, Instant};

use rl_queue::{QuotaState, RateLimiter};
use sqlx::SqlitePool;
use tracing::log::{info, warn};

use super::{
    api::AiEnv,
    multi_statement_classifier::{
        MultiStatementPrompt, MultiStatementPromptGen, MultiStatementPromptResult,
        MultiStatementResultTypes,
    },
    openai::{OpenAiEnv, OpenAiModel}, prompts::{StatementMetaContainer, StatementMeta},
};

/// Runs given prompts and yields results
pub struct PromptRunner<'a, E: AiEnv + 'a> {
    /// Used to set a rate based on the amount of tokens that we have used overall
    token_rate_limiter: RateLimiter,
    env: &'a E,
}

impl<'a, E: AiEnv> PromptRunner<'a, E> {
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
                info!("Running prompt: {}, V{}", prompt.name, prompt.version);
                let response = self.env.send_prompt(&prompt).await?;
                match self
                    .token_rate_limiter
                    .add(response.response.total_tokens as f64)
                {
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
///
/// Will store prompt results in the db
pub async fn run(pool: &SqlitePool) {
    let env = OpenAiEnv::from(OpenAiModel::Gpt35Turbo);

    let prompt_gen = MultiStatementPromptGen::<StatementMetaContainer> {
        batch_size: 5,
        prompt: |stmts| StatementMeta::prompt(&stmts),
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
                let result = runner.run(prompt).await.unwrap();

                result.store(pool).await.unwrap();
            }
            None => {
                async_std::task::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
