use crate::structs::User;
use crate::{error::AppError, pages::charts::ideologies_radar_chart};
use anyhow::Result;

use axum::Extension;

use maud::{html, Markup};
use sqlx::SqlitePool;

use super::base::BaseTemplate;

pub async fn user_page(
    user: User,
    Extension(pool): Extension<SqlitePool>,
    base: BaseTemplate,
) -> Result<Markup, AppError> {
    let ideologies_map = user.num_ideologies(&pool).await?;

    let content = html! {
        div style="padding: 5px 0px; align-self: center;" {
            (ideologies_radar_chart(&ideologies_map).await?)
        }
    };

    Ok(base
       .title("User page")
       .content(content)
       .into())
}
