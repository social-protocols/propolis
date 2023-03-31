use sqlx::SqlitePool;

use crate::structs::{Statement, StatementPrediction};

use super::{
    api::AiEnv,
    prompts::GenericPrompt,
};

async fn find_existing_prediction(
    prediction: &StatementPrediction,
    pool: &SqlitePool,
) -> anyhow::Result<Vec<StatementPrediction>> {
    Ok(sqlx::query_as!(
        StatementPrediction,
        r#"SELECT statement_id, ai_env, prompt_name, prompt_version, prompt_result,
                  completion_tokens, prompt_tokens, total_tokens, timestamp
        FROM statement_predictions
        WHERE statement_id = ? and ai_env = ? and prompt_name = ? and prompt_version = ?"#,
        prediction.statement_id,
        prediction.ai_env,
        prediction.prompt_name,
        prediction.prompt_version
    )
    .fetch_all(pool)
    .await?)
}

pub async fn run<E: AiEnv>(
    s: &Statement,
    prompt: GenericPrompt,
    env: &E,
    pool: &SqlitePool,
) -> anyhow::Result<StatementPrediction> {

    let mut prediction = StatementPrediction {
        statement_id: s.id,
        ai_env: env.name().to_string(),
        prompt_name: prompt.name.clone(),
        prompt_version: prompt.version.into(),
        prompt_result: "".to_string(),
        completion_tokens: 0,
        prompt_tokens: 0,
        total_tokens: 0,
        timestamp: 0,
    };

    let cached_result = find_existing_prediction(&prediction, &pool).await?;
    match cached_result.as_slice() {
        [ref cached_prediction, ..] => {
            prediction = cached_prediction.clone();
        }
        [] => {
            let response = env.send_prompt(&prompt).await?;
            prediction.prompt_result = response.content;
            prediction.completion_tokens = response.completion_tokens;
            prediction.prompt_tokens = response.prompt_tokens;
            prediction.total_tokens = response.total_tokens;
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

            prediction = find_existing_prediction(&prediction, &pool)
                .await?
                .first()
                .unwrap()
                .clone();
        }
    }

    // Ok(serde_json::to_string_pretty(&prediction)
    //     .expect("Serialization of StatementPrediction failed"))

    Ok(prediction)
}
