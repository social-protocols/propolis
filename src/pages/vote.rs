use crate::{auth::User, error::Error, next_statement::redirect_to_next_statement, db::UserQueries};

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
    user.vote(vote.statement_id, vote.vote, &pool).await?;

    Ok(redirect_to_next_statement(Some(user), Extension(pool)).await)
}
