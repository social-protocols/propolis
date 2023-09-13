use crate::error::AppError;
use crate::structs::User;
use crate::structs::Vote;

use anyhow::Result;

use axum::{response::IntoResponse, Extension, Form};
use http::StatusCode;
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use crate::pages::statement::next_statement_id;

#[derive(Deserialize)]
pub struct VoteForm {
    statement_id: i64,
    vote: Vote,
}

pub async fn vote_post(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(vote_form): Form<VoteForm>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::get_or_create(&cookies, &pool).await?;

    user.vote(vote_form.statement_id, vote_form.vote, &pool)
        .await?;

    match vote_form.vote {
        Vote::Yes | Vote::No | Vote::Skip => {
            let next_statement_id = next_statement_id(Some(user), &pool).await?;
            let redirect_url = match next_statement_id {
                Some(statement_id) => format!("/statement/{statement_id}"),
                None => "/statement".to_string(),
            };

            Ok((StatusCode::OK, [("HX-Location", redirect_url)]))
        }
    }
}
