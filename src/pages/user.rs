use crate::error::AppError;
use crate::structs::User;
use anyhow::Result;

use axum::Extension;

use maud::{html, Markup};
use sqlx::SqlitePool;

use super::base::BaseTemplate;

#[cfg(feature = "with_predictions")]
pub async fn ideology_stats(user: User, pool: &SqlitePool) -> Result<Markup, AppError> {
    let ideologies_map = user.ideology_stats_map(pool).await?;
    let bfp_traits_map = user.bfp_traits_map(pool).await?;
    Ok(html! {
        div style="padding: 5px 0px; align-self: center;" {
            (crate::pages::charts::user_radar_chart("Ideologies", &ideologies_map)?)
        }
        div style="padding: 5px 0px; align-self: center;" {
            (crate::pages::charts::user_radar_chart("BFP Traits", &bfp_traits_map)?)
        }
    })
}

#[cfg(not(feature = "with_predictions"))]
pub async fn ideology_stats(_user: User, _pool: &SqlitePool) -> Result<Markup, AppError> {
    Ok(html! {})
}

pub async fn user_page(
    user: User,
    Extension(pool): Extension<SqlitePool>,
    base: BaseTemplate,
) -> Result<Markup, AppError> {
    Ok(base
        .title("User page")
        .content(ideology_stats(user, &pool).await?)
        .into())
}
