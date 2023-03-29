use axum::{extract::Path, response::IntoResponse, Extension};
use maud::html;
use sqlx::SqlitePool;

use crate::{
    db::get_statement,
    error::Error,
    prediction::{
        openai::{OpenAiEnv, OpenAiModel},
        prediction::run,
        prompts::{bfp, statement_category, statement_ideology},
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

    let bfp = run(
        &statement,
        bfp(&statement),
        OpenAiEnv::from(OpenAiModel::Gpt35Turbo),
        &pool,
    )
    .await?;
    let category = run(
        &statement,
        statement_category(&statement),
        OpenAiEnv::from(OpenAiModel::Gpt35Turbo),
        &pool,
    )
    .await?;
    total_tokens += bfp.total_tokens;
    total_tokens += category.total_tokens;

    let mut ideology_prompt_result = "n/a".to_string();

    if category.prompt_result == "political" {
        let ideology = run(
            &statement,
            statement_ideology(&statement),
            OpenAiEnv::from(OpenAiModel::Gpt35Turbo),
            &pool,
        )
        .await?;
        ideology_prompt_result = ideology.prompt_result;
        total_tokens += ideology.total_tokens;
    }

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
            { "BFP trait: " }
            code { (bfp.prompt_result) }
        }
        p {
            { "Statement category: " }
            code { (category.prompt_result) }
        }
        p {
            { "Statement ideology: " }
            code { (ideology_prompt_result) }
        }
    };
    Ok(content)
}
