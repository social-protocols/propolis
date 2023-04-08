use serde::Serialize;
use sqlx::SqlitePool;
use tracing::debug;

use crate::structs::{Statement, StatementPrediction};

use ai_prompt::api::{AiMessage, AiPrompt, PromptResponse};

use propolis_datas::apikey::ApiKey;

/// Helper trait to specify which other traits a type must fulfil in order to be used as a result type
/// of a prompt.
pub trait MultiStatementResultTypes:
    IntoIterator<Item = String> + Clone + Serialize + TryFrom<String, Error = anyhow::Error>
{
}
impl<T> MultiStatementResultTypes for T where
    T: IntoIterator<Item = String> + Clone + Serialize + TryFrom<String, Error = anyhow::Error>
{
}

/// A prompt specific to handling multiple statements at the same time
// FIXME: How can I specify where to store this? e.g. if I want to store embeddings?
//        Probably we want to use a different Prompt type with a different endpoint as well
pub struct MultiStatementPrompt<R: MultiStatementResultTypes> {
    /// Name of the prompt to disambiguate it from others
    pub name: String,
    /// Version of the prompt (newer version invalidates the cache)
    pub version: u16,
    /// Content to send for prediction
    pub primer: Vec<AiMessage>,
    /// Handler for the prediction result
    pub handler: fn(String) -> anyhow::Result<R>,
    /// The statements that this prompt is for
    pub stmts: Vec<Statement>,
}

/// Container for the result of a prediction
#[derive(Serialize)]
pub struct MultiStatementPromptResult<R: MultiStatementResultTypes> {
    /// Contains just response with stats
    pub response: PromptResponse,
    /// Contains the original statement ids used for the prompt
    pub stmts: Vec<Statement>,
    /// Contains the actual result struct after e.g. parsing
    pub result: R,
}

impl<R: MultiStatementResultTypes> MultiStatementPromptResult<R> {
    pub async fn store(&self, api_key: &ApiKey, pool: &SqlitePool) -> anyhow::Result<()> {
        let mut predictions: Vec<StatementPrediction> = vec![];
        // FIXME: Can we somehow get rid of the .clone() calls here?
        let num_stmts = self.result.clone().into_iter().count() as i64;

        for (i, stmt) in self.result.clone().into_iter().enumerate() {
            predictions.push(StatementPrediction {
                statement_id: self.stmts[i].id,
                ai_env: self.response.env_info.to_owned().into(),
                prompt_name: self.response.prompt_info.to_owned().name,
                prompt_version: self.response.prompt_info.version as i64,
                prompt_result: stmt,
                completion_tokens: self.response.completion_tokens / num_stmts,
                prompt_tokens: self.response.prompt_tokens / num_stmts,
                total_tokens: self.response.total_tokens / num_stmts,
                created: 0,
            });
        }

        debug!(
            "Inserting {} entries into statement_predictions table",
            predictions.len()
        );
        for prediction in predictions {
            sqlx::query!(
                r#"INSERT INTO statement_predictions
                (statement_id, ai_env, prompt_name, prompt_version,
                 prompt_result, completion_tokens, prompt_tokens, api_key_id)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
                prediction.statement_id,
                prediction.ai_env,
                prediction.prompt_name,
                prediction.prompt_version,
                prediction.prompt_result,
                prediction.completion_tokens,
                prediction.prompt_tokens,
                api_key.id,
            )
            .execute(pool)
            .await?;
        }
        Ok(())
    }
}

/// Used to generate prompts and handle the result
pub struct MultiStatementPromptGen<'a, R: MultiStatementResultTypes> {
    /// Amount of statements to include in the prompt
    pub batch_size: u8,
    /// Fn taking a batch of statements and yielding a prompt to run
    pub prompt: fn(Vec<Statement>) -> MultiStatementPrompt<R>,
    /// Used for database access to e.g. find next statements to run the prompt on
    pub pool: &'a SqlitePool,
}

impl<R: MultiStatementResultTypes> AiPrompt for MultiStatementPrompt<R> {
    type PromptResult = MultiStatementPromptResult<R>;

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn version(&self) -> u16 {
        self.version
    }

    fn primer(&self) -> Vec<AiMessage> {
        self.primer.clone()
    }

    fn handle_response(&self, response: PromptResponse) -> anyhow::Result<Self::PromptResult> {
        let result = (self.handler)(response.content.clone())?;
        Ok(MultiStatementPromptResult::<R> {
            response,
            result,
            stmts: self.stmts.to_owned(),
        })
    }
}

impl<'a, R: MultiStatementResultTypes> MultiStatementPromptGen<'a, R> {
    /// Returns the next batch of statements to predict
    pub async fn next_batch(&self) -> anyhow::Result<Vec<Statement>> {
        // -- create a dummy prompt so we can figure out for which (name, version) pair to look for --
        let dummy_statement = Statement {
            id: 0,
            text: "".into(),
        };
        let dummy_prompt = (self.prompt)(vec![dummy_statement]);

        // -- find those statements for which a prediction is missing --
        let stmts = sqlx::query_as!(
            Statement,
            "SELECT id,text FROM statements WHERE
id NOT IN
  (SELECT statement_id
   FROM statement_predictions
   WHERE
     prompt_name = ? AND
     prompt_version = ?
)
LIMIT ?",
            dummy_prompt.name,
            dummy_prompt.version,
            self.batch_size,
        )
        .fetch_all(self.pool)
        .await?;
        if !stmts.is_empty() {
            debug!(
                "Next batch len for {} V{}: {}",
                dummy_prompt.name,
                dummy_prompt.version,
                stmts.len()
            );
        }
        Ok(stmts)
    }

    /// Returns a prompt for the next batch of statements
    pub async fn next_prompt(&self) -> anyhow::Result<Option<MultiStatementPrompt<R>>> {
        let batch = self.next_batch().await?;
        if !batch.is_empty() {
            let prompt = (self.prompt)(batch);

            Ok(Some(prompt))
        } else {
            Ok(None)
        }
    }
}
