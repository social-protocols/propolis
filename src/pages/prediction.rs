use axum::{extract::Path, response::IntoResponse, Extension};
use maud::html;
use sqlx::SqlitePool;

use crate::{
    db::get_statement,
    error::Error,
    prediction::{
        openai::{OpenAiEnv, OpenAiModel},
        prediction::run,
        prompts::{bfp, statement_category},
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

    let content = html! {
        p { (format!("Statement ({}):", statement_id.unwrap_or(0))) }
        p { (statement_text) }
        p { "BFP trait:"}
        pre { (bfp) }
        p { "Statement category:"}
        pre { (category) }
    };
    Ok(content)
}
