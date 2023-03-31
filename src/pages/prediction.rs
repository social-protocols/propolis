use axum::{extract::Path, response::IntoResponse, Extension};
use maud::html;
use sqlx::SqlitePool;

use crate::{
    db::get_statement,
    error::Error,
    prediction::{
        openai::{OpenAiEnv, OpenAiModel},
        prediction::run,
        prompts::{bfp, multi_statement_predictor, statement_category, statement_ideology},
    },
    structs::Statement,
};

pub async fn prediction_page(
    Extension(pool): Extension<SqlitePool>,
    Path(statement_id): Path<i64>,
) -> Result<impl IntoResponse, Error> {
    let statement: Option<Statement> = get_statement(statement_id, &pool).await.ok();
    let statement_id = statement.as_ref().map_or(None, |s| Some(s.id));
    let statement_text = statement
        .as_ref()
        .map_or("-".to_string(), |s| s.text.clone());
    let statement = &statement.expect("No such statement");

    let mut total_tokens = 0;

    let env = OpenAiEnv::from(OpenAiModel::Gpt35Turbo);
    let stmts = vec![statement];
    let pred = run(
        &statement,
        multi_statement_predictor(stmts.as_slice()),
        &env,
        &pool,
    )
    .await?;
    total_tokens += pred.total_tokens;

    let content = html! {
        p {
            { (format!("Statement ({}): ", statement_id.unwrap_or(0))) }
            code {  (statement_text) }
        }
        p {
            { "Total tokens: " }
            code { (total_tokens) }
        }
        p {
            { "Result: " }
            pre { (pred.prompt_result) }
        }
    };
    Ok(content)
}
