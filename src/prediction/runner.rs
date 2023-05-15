use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::anyhow;
use propolis_datas::embedding::Embedding;
use rand::seq::SliceRandom;
use tracing::log::error;

use crate::opts::PredictionOpts;
use crate::prediction::embedding::{EmbeddingsRunner, StatementSelector};

use propolis_datas::apikey::{ApiKey, TransientApiKey};
use propolis_datas::statement::StatementFlag;
use propolis_utils::StringExt;
use rl_queue::{QuotaState, RateLimiter};
use sqlx::SqlitePool;
use tracing::{
    debug,
    log::{info, warn},
};

use crate::structs::Statement;

use super::{
    multi_statement_classifier::{
        MultiStatementPrompt, MultiStatementPromptGen, MultiStatementPromptResult,
        MultiStatementResultTypes,
    },
    prompts::{StatementMeta, StatementMetaContainer},
};
use ai_prompt::{
    api::{AiEnv, CheckResult},
    openai::{OpenAiEnv, OpenAiModel},
};

/// Runs given prompts and yields results
pub struct PromptRunner<'a, E: AiEnv + 'a> {
    /// Used to set a rate based on the amount of tokens that we have used overall
    token_rate_limiter: RateLimiter,
    /// Used to set a rate based on how many API calls were done
    api_calls_rate_limiter: RateLimiter,
    env: &'a E,
}

#[derive(Debug)]
pub enum PromptRunnerError {
    CheckFailed,
    Anyhow(anyhow::Error),
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, PromptRunnerError>`. That way you don't need to do that manually.
impl<E> From<E> for PromptRunnerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::Anyhow(err.into())
    }
}

impl<'a, E: AiEnv> PromptRunner<'a, E> {
    /// Run the given prompt and return the result
    pub async fn run<R>(
        &mut self,
        prompt: &MultiStatementPrompt<R>,
    ) -> anyhow::Result<MultiStatementPromptResult<R>, PromptRunnerError>
    where
        R: MultiStatementResultTypes,
    {
        self.token_rate_limiter.block_until_ok().await;
        self.api_calls_rate_limiter.block_until_ok().await;

        self.api_calls_rate_limiter.add(1);

        info!("Running prompt: {}, V{}", prompt.name, prompt.version);
        if let CheckResult::Flagged(err) = self.env.check_prompt(prompt).await? {
            debug!("Prompt failed check: {:?}", err);
            return Err(PromptRunnerError::CheckFailed);
        }
        let response = self.env.send_prompt(prompt).await?;
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

/// Updates the flags on the particular statement, given that they failed the moderation check
///
/// In particular, this will initially flag multiple statements with StatementFlagState::MaybeFlagged
/// and singly predicted statements with StatementFlagState::Flagged.
/// Statements that are flagged via MaybeFlagged, will later be predicted individually.
pub async fn update_failing_statement_flags(
    stmts: &[Statement],
    pool: &mut SqlitePool,
) -> anyhow::Result<()> {
    use propolis_datas::statement::{FlagCategoryContainer, StatementFlagState};

    debug!("Updating statement_flags for failed statements");
    let num_stmts = stmts.len();
    for stmt in stmts {
        match StatementFlag::by_statement_id(pool, stmt.id).await.unwrap() {
            Some(flag) => {
                let mut newflag = flag.clone();
                let flagstate = match num_stmts {
                    1 => StatementFlagState::Flagged,
                    _ => StatementFlagState::MaybeFlagged,
                };
                debug!("Setting flagstate to: {:?}", flagstate);
                newflag.state = flagstate;
                newflag.update(pool).await?;
            }
            None => {
                let flagstate = match num_stmts {
                    1 => StatementFlagState::Flagged,
                    _ => StatementFlagState::MaybeFlagged,
                };
                debug!("Setting flagstate to: {:?}", flagstate);
                StatementFlag::create(pool, stmt.id, flagstate, FlagCategoryContainer::Empty)
                    .await?;
            }
        }
    }
    Ok(())
}

/// Used to select next key to use for requests
pub struct ApiKeySelector {
    pub keys: HashMap<String, ApiKey>,
}

impl ApiKeySelector {
    /// Create an instance, creating keys in the DB (for stats only) if necessary
    pub async fn create(opts: &PredictionOpts, pool: &mut SqlitePool) -> anyhow::Result<Self> {
        let mut keys: HashMap<String, ApiKey> = HashMap::new();
        let mut raw_keys: Vec<String> = opts.openai_api_keys.to_owned();
        if let Some(rk) = &opts.openai_api_key {
            raw_keys.push(rk.into());
        }
        for raw_key in raw_keys {
            let rkey = TransientApiKey::Raw(raw_key.to_owned());
            let key = ApiKey::get_or_create(pool, &rkey, None::<String>).await?;
            keys.insert(raw_key, key);
        }
        Ok(Self { keys })
    }

    /// Yield next key to use
    pub fn next(&self) -> anyhow::Result<(String, ApiKey)> {
        let raw_key = self
            .keys
            .keys()
            .collect::<Vec<&String>>()
            .choose(&mut rand::thread_rng())
            .ok_or(anyhow!("Unable to select any API key."))?
            .to_string();
        let api_key = self.keys.get(&raw_key).unwrap();
        Ok((raw_key, api_key.to_owned()))
    }
}

/// Setup continuous prompt generation and runner in an async loop
///
/// Will store prompt results in the db
pub async fn run(opts: &crate::opts::PredictionOpts, pool: &mut SqlitePool) {
    let key_selector = ApiKeySelector::create(opts, pool)
        .await
        .expect("Unable to setup key selection.");
    let env = OpenAiEnv::from(OpenAiModel::Gpt35Turbo);

    info!("API keys loaded: {}", key_selector.keys.len());
    info!("Prediction environment: {:?}", env);

    let mut pool2 = pool.to_owned();
    let prompt_gen = MultiStatementPromptGen::<StatementMetaContainer> {
        batch_size: 5,
        prompt: |stmts| StatementMeta::prompt(&stmts),
        pool,
    };

    let mut erunner = EmbeddingsRunner {
        token_rate_limiter: RateLimiter::new(
            opts.tokens_per_duration as f64,
            Duration::from_secs(opts.tokens_seconds_per_duration),
        ),
        api_calls_rate_limiter: RateLimiter::new(
            opts.api_calls_per_duration as f64,
            Duration::from_secs(opts.api_calls_seconds_per_duration),
        ),
        env: &env,
    };
    let selector = StatementSelector {};

    let mut runner = PromptRunner {
        token_rate_limiter: RateLimiter::new(
            opts.tokens_per_duration as f64,
            Duration::from_secs(opts.tokens_seconds_per_duration),
        ),
        api_calls_rate_limiter: RateLimiter::new(
            opts.api_calls_per_duration as f64,
            Duration::from_secs(opts.api_calls_seconds_per_duration),
        ),
        env: &env,
    };
    loop {
        async_std::task::sleep(Duration::from_secs(1)).await;

        // select data
        let prompt = prompt_gen.next_prompt().await.unwrap_or_else(|err| {
            error!("next_prompt failed: {}", err);
            None
        });

        let embed_stmts = match selector.next_for_embedding(pool).await {
            Ok(stmts) if !stmts.is_empty() => Some(stmts),
            Ok(_) => None,
            Err(err) => {
                error!("Unable to select next statements for embedding: {:?}", err);
                None
            }
        };

        // short-circuit loop in case no data
        if let (None, None) = (&prompt, &embed_stmts) {
            continue;
        }

        // select key
        let (raw_key, api_key) = key_selector.next().expect("Unable to select key");
        debug!(
            "Using key ({}): {}",
            api_key.id,
            raw_key.as_str().shortify(2, 4, "..")
        );
        ai_prompt::openai::set_key(raw_key);

        if let Some(prompt) = prompt {
            // run prompt
            match runner.run(&prompt).await {
                Ok(result) => {
                    if let Err(err) = result.store(&api_key, pool).await {
                        error!("storing result failed: {err}");
                    };
                }
                Err(PromptRunnerError::CheckFailed) => {
                    match update_failing_statement_flags(&prompt.stmts, &mut pool2).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Unable to update statement flags: {}", err)
                        }
                    }
                }
                Err(err) => {
                    error!("running prompt failed: {:?}", err);
                }
            };
        }

        if let Some(embed_stmts) = embed_stmts {
            // run embeddings
            let len = embed_stmts.len();
            info!("Embedding {len} statements");
            let embeddings = match erunner
                .run(
                    &embed_stmts
                        .iter()
                        .map(|stmt| stmt.text.as_str())
                        .collect::<Vec<&str>>(),
                )
                .await
            {
                Ok(e) => Some(e),
                Err(err) => {
                    error!("running for embeddings failed: {:?}", err);
                    None
                }
            };

            if let Some((embeddings, total_tokens)) = embeddings {
                for (i, embedding) in embeddings.iter().enumerate() {
                    match Embedding::create(
                        &mut pool2,
                        embed_stmts[i].id,
                        embedding.values.clone(),
                        total_tokens.into(),
                        api_key.id,
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(err) => {
                            error!("storing of embedding failed: {:?}", err);
                        }
                    }
                }
            }
        }
    }
}
