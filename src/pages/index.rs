use crate::{
    auth::{ensure_auth, User},
    next_statement::{next_statement_for_user, random_statement_id},
};

use axum::{response::Redirect, Extension, Form};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

pub async fn index(
    existing_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
) -> Redirect {
    let statement_id: Option<i64> = match existing_user {
        Some(user) => next_statement_for_user(user.id, &pool).await,
        None => random_statement_id(&pool).await,
    };

    match statement_id {
        Some(id) => Redirect::to(format!("/statement/{}", id).as_str()),
        None => Redirect::to("/statement/0"), // TODO
    }
}

#[derive(Deserialize)]
pub struct VoteForm {
    statement_id: i64,
    vote: i32,
}

pub async fn vote(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(vote): Form<VoteForm>,
) -> Redirect {
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

    Redirect::to("/")
}
