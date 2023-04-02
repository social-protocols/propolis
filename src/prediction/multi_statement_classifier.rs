use serde::Serialize;
use sqlx::SqlitePool;
use tracing::debug;

use crate::structs::{Statement, StatementPrediction};

use super::api::{AiMessage, AiPrompt, PromptResponse};

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
    pub handler: fn(String) -> R,
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
    pub async fn store(&self, pool: &SqlitePool) -> anyhow::Result<()> {
        let mut predictions: Vec<StatementPrediction> = vec![];
        let num_stmts = 1;

        for (i, stmt) in self.result.clone().into_iter().enumerate() {
            predictions.push(StatementPrediction {
                statement_id: self.stmts[i].id,
                ai_env: self.response.env_info.to_owned().into(),
                prompt_name: self.response.prompt_info.to_owned().name,
                prompt_version: self.response.prompt_info.version as i64,
                prompt_result: stmt.into(),
                completion_tokens: self.response.completion_tokens / num_stmts,
                prompt_tokens: self.response.prompt_tokens / num_stmts,
                total_tokens: self.response.total_tokens / num_stmts,
                timestamp: 0,
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
                 prompt_result, completion_tokens, prompt_tokens)
                VALUES (?, ?, ?, ?, ?, ?, ?)"#,
                prediction.statement_id,
                prediction.ai_env,
                prediction.prompt_name,
                prediction.prompt_version,
                prediction.prompt_result,
                prediction.completion_tokens,
                prediction.prompt_tokens,
            )
            .execute(pool)
            .await?;
        }
        Ok(())
    }
}

impl<R: MultiStatementResultTypes> MultiStatementPrompt<R> {
    /// Gives a prompt that computes various meta information on the passed statements
    // TODO: move to prompts.rs
    pub fn statement_meta(stmts: &[Statement]) -> Self {
        let mut stmts_s = String::from("");
        for s in stmts {
            stmts_s += format!("{}: {}", s.id, s.text).as_str();
        }
        MultiStatementPrompt {
            name: "statement_meta".into(),
            version: 5,
            handler: |s| {
                let s_without_header = s.trim().splitn(2, "\n").nth(1).unwrap_or("").to_string();
                let target_delim_count = 7;
                s_without_header
                    .split("\n")
                    // -- fixup delimiter count, since the ai does not reliably do that --
                    .map(|s| -> String {
                        let s = s.to_string();
                        let delim_count = s.match_indices("|").collect::<Vec<_>>().len();
                        if delim_count < target_delim_count {
                            s + "|".repeat(target_delim_count - delim_count).as_str().into()
                        } else if delim_count > target_delim_count {
                            s.strip_suffix("|".repeat(delim_count - target_delim_count).as_str())
                                .unwrap()
                                .into()
                        } else {
                            s
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
                    // -- finally try to return an R (result type) --
                    .try_into()
                    .expect("Unable to extract from string")
            },
            primer: vec![
                AiMessage::system(
                    "
You will be given multiple statements, each starting on their own line,
and your task is to determine whether the statement falls into the category
of politics or personal statements. In the case of it being a political category,
give which political ideologies (e.g., liberalism, conservatism, socialism)
each quote aligns with the most.
In the case of it being a personal category, give the big five personality traits instead.

In addition, also output up to three topic tags. The output should be a csv table with empty values as \"-\".
All cells should be followed by a strength score (w=weak, s=strong) after a \":\" delimiter.
If you are not sure, use \"-\"",
                ),
                AiMessage::user(
                    "
1. The global economy is at risk of recession due to the trade war and uncertainty it creates.
2. In clubs kann man hervorragend neue Freunde kennenlernen",
                ),
                AiMessage::assistant(
                    "
num|category|label1|label2|label3|tag1|tag2|tag3
1|politics|neoliberalism:s|conservatism:w|socialism:w|global economy:s|trade war:s|uncertainty:s
2|personal|extraversion:s|openness:w|agreeableness:s|clubs:s|friendship:s|socializing:w
",
                ),
                AiMessage::user(format!("{}", stmts_s).as_str()),
            ],
            stmts: stmts.to_vec(),
        }
    }
}

/// Used to generate prompts and handle the result
///
/// Will cache results in the db based on the given prompt
pub struct MultiStatementPromptGen<'a, R: MultiStatementResultTypes> {
    /// Amount of statements to include in the prompt
    pub batch_size: u8,
    /// Fn taking a batch of statements and yielding a prompt to run
    pub prompt: fn(Vec<Statement>) -> MultiStatementPrompt<R>,
    /// Used for database access to e.g. find next statements to run the prompt on
    pub pool: &'a SqlitePool,
}

// TODO: this should probably go into prompts.rs
/// A single row of the result that we get back via the multi_statement_predictor
// #[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
// pub struct StatementMeta {
//     pub category: String,
//     pub label1: String,
//     pub label2: String,
//     pub label3: String,
//     pub tag1: String,
//     pub tag2: String,
//     pub tag3: String,
// }

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

    fn handle_response(&self, response: PromptResponse) -> Self::PromptResult {
        let result: R = (self.handler)(response.content.clone()).try_into().unwrap();
        MultiStatementPromptResult::<R> {
            response,
            result,
            stmts: self.stmts.to_owned(),
        }
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
        if stmts.len() > 0 {
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
        if batch.len() > 0 {
            let prompt = (self.prompt)(batch);

            Ok(Some(prompt))
        } else {
            Ok(None)
        }
    }
}
