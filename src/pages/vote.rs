use crate::structs::User;
use crate::{error::Error, structs::Vote};

use axum::{response::Redirect, Extension, Form};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use super::index::redirect_to_next_statement;

#[derive(Deserialize)]
pub struct VoteForm {
    statement_id: i64,
    vote: Vote,
}

pub async fn vote(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(vote_form): Form<VoteForm>,
) -> Result<Redirect, Error> {
    let user = User::get_or_create(&cookies, &pool).await?;

    user.vote(vote_form.statement_id, vote_form.vote, &pool)
        .await?;

    match vote_form.vote {
        Vote::Yes | Vote::No | Vote::Skip => {
            Ok(redirect_to_next_statement(Some(user), Extension(pool)).await?)
        }
        Vote::ItDepends => Ok(Redirect::to(
            format!("/new?target={}", vote_form.statement_id).as_str(),
        )),
    }
}
