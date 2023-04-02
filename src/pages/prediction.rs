use axum::{extract::Path, response::IntoResponse, Extension};
use maud::html;
use sqlx::SqlitePool;

use crate::{
    db::get_statement,
    error::Error,
    structs::Statement,
};

pub async fn prediction_page(
    Extension(pool): Extension<SqlitePool>,
    Path(statement_id): Path<i64>,
) -> Result<impl IntoResponse, Error> {
    let statement: Option<Statement> = get_statement(statement_id, &pool).await.ok();
    let _statement_id = statement.as_ref().map_or(None, |s| Some(s.id));
    let statement_text = statement
        .as_ref()
        .map_or("-".to_string(), |s| s.text.clone());

    let content = html! {
        p { (statement_text) }
    };
    Ok(content)
}
