use crate::db::random_statement_id;
use crate::error::AppError;
use crate::structs::User;
use crate::structs::Vote;

use anyhow::Result;

use axum::response::Redirect;
use axum::response::Response;
use axum::{response::IntoResponse, Extension, Form};
use http::StatusCode;
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use super::base::BaseTemplate;
use super::statement::history;

pub async fn next_statement_id(
    existing_user: Option<User>,
    pool: &SqlitePool,
) -> Result<Option<i64>> {
    Ok(match existing_user {
        Some(user) => user.next_statement_for_user(pool).await?,
        None => random_statement_id(pool).await?,
    })
}

pub async fn vote(
    existing_user: Option<User>,
    maybe_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
    base: BaseTemplate,
) -> Result<Response, AppError> {
    let statement_id = next_statement_id(existing_user, &pool).await?;

    Ok(match statement_id {
        Some(id) => Redirect::to(format!("/statement/{id}").as_str()).into_response(),
        None => base
            .content(history(&maybe_user, &pool).await?)
            .render()
            .into_response(),
    })
}

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
                None => "/vote".to_string(),
            };

            Ok((StatusCode::OK, [("HX-Location", redirect_url)]))
        }
    }
}
