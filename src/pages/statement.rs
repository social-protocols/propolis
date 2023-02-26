use super::base::{get_base_template, BaseTemplate};
use crate::{db::get_statement, error::Error, structs::Statement};

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
) -> Result<Html<String>, Error> {
    let statement = get_statement(statement_id, &pool).await?;

    let template = StatementTemplate {
        base: get_base_template(cookies, Extension(pool)),
        statement: &statement,
    };

    Ok(Html(template.render().unwrap()))
}
