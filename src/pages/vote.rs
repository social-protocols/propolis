use crate::structs::User;
use crate::{error::Error, structs::Vote};

use axum::extract::Path;
use axum::response::Html;
use axum::{response::IntoResponse, Extension, Form};
use http::StatusCode;
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use super::index::next_statement_id;
use super::statement::votes;

#[derive(Deserialize)]
pub struct VoteForm {
    statement_id: i64,
    vote: Vote,
}

pub async fn vote(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(vote_form): Form<VoteForm>,
) -> Result<impl IntoResponse, Error> {
    let user = User::get_or_create(&cookies, &pool).await?;

    user.vote(vote_form.statement_id, vote_form.vote, &pool)
        .await?;

    match vote_form.vote {
        Vote::Yes | Vote::No | Vote::Skip => {
            let id = next_statement_id(Some(user), Extension(pool.to_owned())).await?;
            let redirect_url = match id {
                Some(id) => Some(format!("/statement/{}", id)),
                None => None,
            };

            match redirect_url {
                Some(redirect_url) => {
                    let body = votes(Path(vote_form.statement_id), Extension(pool)).await?;
                    Ok((
                        StatusCode::OK,
                        [(
                            "HX-Trigger",
                            format!(
                                r##"{{"delayedRedirectTo": {{"value": "{}"}}}}"##,
                                redirect_url
                            ),
                        )],
                        body,
                    ))
                }
                None => Err(Error::CustomError("No next statement".to_string())),
            }
        }
        Vote::ItDepends => Ok((
            StatusCode::OK,
            [(
                "HX-Redirect",
                format!("/new?target={}", vote_form.statement_id),
            )],
            Html::from(String::from("")),
        )),
    }
}
