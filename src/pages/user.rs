use crate::{pages::charts::ideologies_radar_chart, error::AppError};
use crate::structs::User;
use anyhow::Result;

use axum::Extension;

use http::HeaderMap;
use maud::{html, Markup};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use super::base::base;

pub async fn user_page(
    user: User,
    cookies: Cookies,
    headers: HeaderMap,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Markup, AppError> {
    let ideologies_map = user.num_ideologies(&pool).await?;

    let content = html! {
        div style="padding: 5px 0px; align-self: center;" {
            (ideologies_radar_chart(&ideologies_map).await?)
        }
    };

    let maybe_user = Some(user);

    Ok(base(
        cookies,
        Some("User page".to_string()),
        &maybe_user,
        content,
        &headers,
        None,
    ))
}
