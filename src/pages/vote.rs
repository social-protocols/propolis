use crate::{auth::User, error::Error, next_statement::redirect_to_next_statement};

use axum::{response::Redirect, Extension, Form};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct VoteForm {
    statement_id: i64,
    vote: i32,
}

pub async fn vote(
    user: User,
    Extension(pool): Extension<SqlitePool>,
    Form(vote): Form<VoteForm>,
) -> Result<Redirect, Error> {
    sqlx::query!(
        "INSERT INTO votes (statement_id, user_id, vote)
VALUES (?, ?, ?)
on CONFLICT (statement_id, user_id)
do UPDATE SET vote = excluded.vote",
        vote.statement_id,
        user.id,
        vote.vote
    )
    .execute(&pool)
    .await?;

    sqlx::query!(
        "INSERT INTO vote_history (user_id, statement_id, vote) VALUES (?, ?, ?)",
        user.id,
        vote.statement_id,
        vote.vote
    )
    .execute(&pool)
    .await?;

    sqlx::query!(
        "delete from queue where user_id = ? and statement_id = ?",
        user.id,
        vote.statement_id
    )
    .execute(&pool)
    .await?;

    Ok(redirect_to_next_statement(Some(user), Extension(pool)).await)
}
