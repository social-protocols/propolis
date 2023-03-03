use crate::error::Error;
use crate::structs::User;

use axum::{response::{Redirect, IntoResponse}, Extension, Form};
use http::{header, StatusCode};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use super::index::{redirect_to_next_statement, next_statement_id};

#[derive(Deserialize)]
pub struct VoteForm {
    statement_id: i64,
    vote: i32,
}

pub async fn vote(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(vote): Form<VoteForm>,
) -> Result<impl IntoResponse, Error> {
    let user = User::get_or_create(&cookies, &pool).await?;

    user.vote(vote.statement_id, vote.vote, &pool).await?;

    let id = next_statement_id(Some(user), Extension(pool)).await?;
    let redirect_url = match id {
        Some(id) => Some(format!("/statement/{}", id)),
        None => None,
    };

    match redirect_url {
        Some(redirect_url) => Ok((
            StatusCode::OK,
            [("HX-Redirect", redirect_url)]
        )),
        None => Err(Error::CustomError("No next statement".to_string()))
    }
}
