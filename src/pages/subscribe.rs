use crate::error::Error;
use crate::structs::User;

use axum::{response::IntoResponse, Extension, Form};

use serde::Deserialize;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

#[derive(Deserialize)]
pub struct FollowForm {
    statement_id: i64,
}

pub async fn subscribe(
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    Form(form_data): Form<FollowForm>,
) -> Result<impl IntoResponse, Error> {
    let user = User::get_or_create(&cookies, &pool).await?;
    user.follow(form_data.statement_id, &pool).await?;

    Ok("subscribed")
}
