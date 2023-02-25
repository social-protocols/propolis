use super::base::{get_base_template, BaseTemplate};
use crate::structs::Statement;

use askama::Template;
use axum::{extract::Path, response::Html, Extension};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Template)]
#[template(path = "statement.j2")]
struct StatementTemplate<'a> {
    base: BaseTemplate,
    statement: &'a Option<Statement>,
}

pub async fn statement(
    Path(statement_id): Path<i64>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Html<String> {
    let statement = get_statement(statement_id, &pool).await;

    let template = StatementTemplate {
        base: get_base_template(cookies, Extension(pool)),
        statement: &statement,
    };

    Html(template.render().unwrap())
}

pub async fn get_statement(statement_id: i64, pool: &SqlitePool) -> Option<Statement> {
    sqlx::query_as!(
        Statement,
        // TODO: https://github.com/launchbadge/sqlx/issues/1524
        "SELECT id, text from statements where id = ?",
        statement_id,
    )
    .fetch_optional(pool)
    .await
    .expect("Must be valid")
}
