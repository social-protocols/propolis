use std::time::{Duration, Instant};

use rl_queue::{QuotaState, RateLimiter};
use sqlx::SqlitePool;
use tracing::log::{info, warn};

use super::{
    multi_statement_classifier::{
        MultiStatementPrompt, MultiStatementPromptGen, MultiStatementPromptResult,
        MultiStatementResultTypes,
    },
    prompts::{StatementMeta, StatementMetaContainer},
};
use ai_prompt::{
    api::AiEnv,
    openai::{OpenAiEnv, OpenAiModel},
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
        self.token_rate_limiter.block_until_ok().await;

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

        Ok(response)
    }
}

/// Setup continuous prompt generation and runner in an async loop
///
/// Will store prompt results in the db
#[cfg(feature = "with_predictions")]
pub async fn run(opts: crate::opts::PredictionOpts, pool: &mut SqlitePool) {
    use std::collections::HashMap;

    use rand::seq::SliceRandom;
    use tracing::log::error;

    use propolis_datas::apikey::{ApiKey, TransientApiKey};

    let mut keys: HashMap<String, ApiKey> = HashMap::new();
    let mut raw_keys: Vec<String> = opts.openai_api_keys;
    if let Some(rk) = opts.openai_api_key {
        raw_keys.push(rk);
    }
    for raw_key in raw_keys {
        let rkey = TransientApiKey::Raw(raw_key.to_owned());
        let key = ApiKey::get_or_create(pool, &rkey, None::<String>)
            .await
            .expect("Unable to get_or_create key.");
        keys.insert(raw_key, key);
    }
    let env = OpenAiEnv::from(OpenAiModel::Gpt35Turbo);

    info!("API keys loaded: {}", keys.len());
    info!("Prediction environment: {:?}", env);

    let prompt_gen = MultiStatementPromptGen::<StatementMetaContainer> {
        batch_size: 5,
        prompt: |stmts| StatementMeta::prompt(&stmts),
        pool,
    };

    let mut runner = PromptRunner {
        token_rate_limiter: RateLimiter::new(
            opts.tokens_per_duration as f64,
            Duration::from_secs(opts.seconds_per_duration),
        ),
        env: &env,
    };
    loop {
        let raw_key = keys
            .keys()
            .collect::<Vec<&String>>()
            .choose(&mut rand::thread_rng())
            .expect("Unable to select any API key.")
            .to_string();
        let api_key = keys.get(&raw_key).unwrap();

        let prompt = prompt_gen.next_prompt().await.unwrap_or_else(|err| {
            error!("next_prompt failed: {}", err);
            None
        });

        match prompt {
            Some(prompt) => {
                ai_prompt::openai::set_key(raw_key)
                    .await
                    .expect("Unable to setup openai");

                match runner.run(prompt).await {
                    Ok(result) => {
                        if let Err(err) = result.store(api_key, pool).await {
                            error!("storing result failed: {err}");
                        };
                    }
                    Err(err) => {
                        error!("running prompt failed: {err}");
                    }
                };
            }
            None => {
                async_std::task::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
