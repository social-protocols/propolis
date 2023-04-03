use axum::{extract::Path, response::IntoResponse, Extension};
use maud::html;
use sqlx::SqlitePool;

use crate::{
    db::get_statement, error::Error, prediction::prompts::StatementMeta,
    structs::StatementPrediction,
};

pub async fn prediction_page(
    Extension(pool): Extension<SqlitePool>,
    Path(statement_id): Path<i64>,
) -> Result<impl IntoResponse, Error> {
    let statement = get_statement(statement_id, &pool).await?;

    let pred = sqlx::query_as!(
        StatementPrediction,
        "select
-- see: https://github.com/launchbadge/sqlx/issues/1126 on why this is necessary when using ORDER BY
  statement_id as \"statement_id!\",
  ai_env as \"ai_env!\",
  prompt_name as \"prompt_name!\",
  prompt_version as \"prompt_version!\",
  prompt_result as \"prompt_result!\",
  completion_tokens as \"completion_tokens!\",
  prompt_tokens as \"prompt_tokens!\",
  total_tokens as \"total_tokens!\",
  timestamp as \"timestamp!\"
from statement_predictions
where statement_id = ? order by timestamp desc",
        statement.id
    )
    .fetch_optional(&pool)
    .await?
    .map_or("no prediction yet".to_string(), |s| s.prompt_result.into());

    let pred_formatted = serde_json::to_string_pretty(
        &serde_json::from_str::<StatementMeta>(pred.as_str())?,
    )?;

    let content = html! {
        p { (statement.text) }
        pre { (pred_formatted) }
    };
    Ok(content)
}
