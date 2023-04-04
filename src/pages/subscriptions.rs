use super::base::base;
use crate::error::Error;
use crate::pages::statement_ui::{
    small_statement_piechart, small_statement_predictions, small_statement_vote_fetch,
};
use crate::structs::User;

use crate::{db::get_subscriptions, pages::statement_ui::small_statement_content};

use axum::Extension;
use http::HeaderMap;
use maud::{html, Markup};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

pub async fn subscriptions(
    maybe_user: Option<User>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
    headers: HeaderMap,
) -> Result<Markup, Error> {
    let subscriptions = match &maybe_user {
        Some(user) => get_subscriptions(user, &pool).await?,
        None => Vec::new(),
    };

    let content = html! {
        h1 { "My Subscriptions" }
        @if subscriptions.is_empty() {
            p { "You have not subscribed any statements yet" }
        }
        @for statement in subscriptions {
            div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                (small_statement_predictions(&statement, &pool).await?)
                (small_statement_content(&statement, None, true, &maybe_user, &pool).await?)
                (small_statement_piechart(statement.id, &pool).await?)
                (small_statement_vote_fetch(statement.id, &maybe_user, &pool).await?)
            }
        }
    };
    Ok(base(cookies, None, &maybe_user, content, &headers, None))
}
