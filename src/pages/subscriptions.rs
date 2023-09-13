use crate::pages::base_template::BaseTemplate;

use crate::error::AppError;
use crate::pages::statement_ui::{
    inline_statement_piechart, inline_statement_predictions, inline_statement_vote_fetch,
};
use crate::structs::User;

use crate::{db::get_subscriptions, pages::statement_ui::inline_statement_content};

use axum::Extension;
use maud::{html, Markup};
use sqlx::SqlitePool;

pub async fn subscriptions(
    maybe_user: Option<User>,
    Extension(pool): Extension<SqlitePool>,
    base: BaseTemplate,
) -> Result<Markup, AppError> {
    let subscriptions = match &maybe_user {
        Some(user) => get_subscriptions(user, &pool).await?,
        None => Vec::new(),
    };

    let content = html! {
        h1 class="text-xl mb-4" { "My Subscriptions" }
        @if subscriptions.is_empty() {
            p { "You have not subscribed to any statements yet" }
        }
        @for (i, statement) in subscriptions.iter().enumerate() {
            div data-testid={"subscription-statement-"(i)} class="mb-5 rounded-lg shadow bg-white dark:bg-slate-700 flex " {
                (inline_statement_predictions(statement, &pool).await?)
                (inline_statement_content(statement, None, true, &maybe_user, &pool).await?)
                (inline_statement_piechart(statement.id, &pool).await?)
                (inline_statement_vote_fetch(statement.id, &maybe_user, &pool).await?)
            }
        }
    };
    Ok(base.title("Subscriptions").content(content).render())
}
