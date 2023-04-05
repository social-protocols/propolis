use crate::error::Error;
use crate::structs::User;

use axum::{Extension, response::IntoResponse};

use maud::html;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

pub async fn page(
    _user: Option<User>,
    _cookies: Cookies,
    Extension(_pool): Extension<SqlitePool>,
) -> Result<impl IntoResponse, Error> {

    Ok(html! {
        "your content here"
    })
}
