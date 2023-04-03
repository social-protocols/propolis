use axum::{extract::Path, response::IntoResponse, Extension};
use maud::html;
use sqlx::SqlitePool;

use crate::{db::get_statement, error::Error};

pub async fn prediction_page(
    Extension(pool): Extension<SqlitePool>,
    Path(statement_id): Path<i64>,
) -> Result<impl IntoResponse, Error> {
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
