use crate::error::Error;
use crate::structs::User;

use axum::{Extension, Form};

use maud::{html, Markup};
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
) -> Result<Markup, Error> {
    let user = User::get_or_create(&cookies, &pool).await?;
    user.subscribe(form_data.statement_id, &pool).await?;

    Ok(html! { span style="padding: 0.4em 1em; opacity: 0.5; font-size: 85%" { "subscribed" } })
}
