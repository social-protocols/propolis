use super::base::{get_base_template, BaseTemplate};
use crate::{
    auth::{ensure_auth, logged_in_user},
    next_statement::{next_statement_for_anonymous, next_statement_for_user},
    structs::Statement,
};

use askama::Template;
use axum::{response::Html, Extension, Form};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Template)]
#[template(path = "index.j2")]
struct IndexTemplate<'a> {
    base: BaseTemplate,
    statement: &'a Option<Statement>,
}

pub async fn index(cookies: Cookies, Extension(pool): Extension<SqlitePool>) -> Html<String> {
    let existing_user = logged_in_user(&cookies, &pool).await;

    let statement: Option<Statement> = match existing_user {
        Some(user) => next_statement_for_user(user.id, &pool).await,
        None => next_statement_for_anonymous(&pool).await,
    };

    let template = IndexTemplate {
        base: get_base_template(cookies, Extension(pool)),
        statement: &statement,
    };

    Html(template.render().unwrap())
}

#[derive(Deserialize)]
pub struct VoteForm {
    statement_id: i64,
    vote: i32,
}

pub async fn index_post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(vote): Form<VoteForm>,
) -> Html<String> {
    let user = ensure_auth(&cookies, &pool).await;

    sqlx::query!(
        "INSERT INTO votes (statement_id, user_id, vote) VALUES (?, ?, ?) on conflict ( statement_id, user_id) do update set vote = excluded.vote",
        vote.statement_id,
        user.id,
        vote.vote
    )
    .execute(&pool)
    .await
    .expect("Database problem");

    sqlx::query!(
        "INSERT INTO vote_history (user_id, statement_id, vote) VALUES (?, ?, ?)",
        user.id,
        vote.statement_id,
        vote.vote
    )
    .execute(&pool)
    .await
    .expect("Database problem");

    sqlx::query!(
        "delete from queue where user_id = ? and statement_id = ?",
        user.id,
        vote.statement_id
    )
    .execute(&pool)
    .await
    .expect("Database problem");

    index(cookies, Extension(pool)).await
}
