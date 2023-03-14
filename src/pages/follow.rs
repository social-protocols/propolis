use crate::structs::User;
use crate::{error::Error, structs::Vote};

use axum::extract::{Path, Query};
use axum::{response::IntoResponse, Extension, Form};
use http::StatusCode;
use maud::html;
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use super::index::next_statement_id;
use super::statement::votes;

#[derive(Deserialize)]
pub struct FollowForm {
    statement_id: i64,
}

pub async fn follow(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(form_data): Form<FollowForm>,
) -> Result<impl IntoResponse, Error> {
    let user = User::get_or_create(&cookies, &pool).await?;
    user.follow(form_data.statement_id, &pool).await?;

    Ok(StatusCode::OK)
}
