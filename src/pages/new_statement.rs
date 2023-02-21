use crate::auth::ensure_auth;

use askama::Template;
use axum::{response::{Html, Redirect}, Extension, Form};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;


#[derive(Template)]
#[template(path = "new_statement.j2")]
struct NewStatementTemplate {}

pub async fn new_statement() -> Html<String> {
    let template = NewStatementTemplate {};

    Html(template.render().unwrap())
}

#[derive(Deserialize)]
pub struct AddStatementForm {
    statement_text: String,
}

pub async fn new_statement_post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(add_statement): Form<AddStatementForm>,
) -> Redirect {
    let user = ensure_auth(&cookies, &pool).await;
    // TODO: add statement and author entry in transaction
    let created_statement = sqlx::query!(
        "INSERT INTO statements (text) VALUES (?) RETURNING id",
        add_statement.statement_text
    )
    .fetch_one(&pool)
    .await
    .expect("Database problem");

    let query = sqlx::query!(
        "INSERT INTO authors (user_id, statement_id) VALUES (?, ?)",
        user.id,
        created_statement.id
    )
    .execute(&pool)
    .await;
    query.expect("Database problem");

    let query = sqlx::query!(
        "INSERT INTO queue (user_id, statement_id) VALUES (?, ?)",
        user.id,
        created_statement.id
    )
    .execute(&pool)
    .await;
    query.expect("Database problem");

    Redirect::to("/")
}

