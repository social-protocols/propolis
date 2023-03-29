use axum::{extract::Path, response::IntoResponse, Extension};
use maud::html;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use crate::{
    db::get_statement,
    error::Error,
    prediction::{
        openai::{OpenAiEnv, OpenAiModel},
        prediction::bfp,
    },
    structs::{Statement, User},
};

pub async fn prediction_page(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Path(statement_id): Path<i64>,
    maybe_user: Option<User>,
) -> Result<impl IntoResponse, Error> {
    let statement: Option<Statement> = get_statement(statement_id, &pool).await.ok();
    let statement_text = statement.map_or("-".to_string(), |s| s.text.clone());
    let bfp_result = match bfp(&statement_text, OpenAiEnv::from(OpenAiModel::Gpt35_turbo)).await {
        Ok(result) => result,
        Err(err) => {
            println!("GPT ERROR: {:?}", err);
            err.to_string()
        }
    };

    let content = html! {
        p { "Statement:" }
        p { (statement_text) }
        p { "BFP trait:"}
        p { (bfp_result)}
    };
    Ok(content)
}
