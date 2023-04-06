use anyhow::Result;
use axum::{extract::Path, Extension};
use maud::{html, Markup};
use sqlx::SqlitePool;

use crate::{db::get_statement, error::AppError};

pub async fn prediction_page(
    Extension(pool): Extension<SqlitePool>,
    Path(statement_id): Path<i64>,
) -> Result<Markup, AppError> {
    let statement = get_statement(statement_id, &pool).await?;

    let meta = statement.get_meta(&pool).await?;
    let pred_formatted = match meta {
        Some(meta) => serde_json::to_string_pretty(&meta)?,
        None => "".into(),
    };

    let content = html! {
        p { (statement.text) }
        pre { (pred_formatted) }
    };
    Ok(content)
}
