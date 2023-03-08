use super::base::{get_base_template, GenericViewTemplate};
use crate::error::Error;
use crate::structs::User;
use crate::util::human_relative_time;

use axum::{response::Html, Extension};
use maud::html;
use sqlx::SqlitePool;
use tower_cookies::Cookies;

pub async fn history(
    maybe_user: Option<User>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Html<String>, Error> {
    let history = match maybe_user {
        Some(user) => user.vote_history(&pool).await?,
        None => Vec::new(),
    };

    let content = html! {
        h1 {
            "Recent Votes"
        }
        @if history.len() == 0 {
            p { "You have not submitted any votes yet" }
        }
        @for item in history {
            div class="card info" {
                p {
                    a href=(format!("/statement/{}", item.statement_id))  {
                        (item.statement_text)
                    }
                }
                p {
                    "Your Vote: "
                    @if item.vote == 1 {
                        "Yes"
                    } @else if item.vote == -1 {
                        "No"
                    }
                }
                p {
                    (human_relative_time(&item.vote_timestamp))
                }
                input type="hidden" value=(item.statement_id);
                a href=(format!("/new?target={}", item.statement_id)) {
                    "Reply"
                }
            }
        }
    };

    let base = get_base_template(cookies, Extension(pool));
    GenericViewTemplate {
        base,
        content: content.into_string().as_str(),
        title: None,
    }
    .into()
}
