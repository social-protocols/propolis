use super::base::base;
use crate::db::get_submissions;
use crate::error::Error;
use crate::structs::User;
use crate::util::human_relative_time;

use axum::Extension;
use maud::{html, Markup};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

pub async fn submissions(
    maybe_user: Option<User>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Markup, Error> {
    let submissions = match &maybe_user {
        Some(user) => get_submissions(&user, &pool).await?,
        None => Vec::new(),
    };

    let content = html! {
        h1 { "Your Submissions" }
        @if submissions.len() == 0 {
            p { "You have not submitted any statements yet" }
        }
        @for item in submissions {
            div.card.info {
                p {
                    a href=(format!("/statement/{}", item.statement_id))  {
                        (item.statement_text)
                    }
                }
                @if item.vote != 0 {
                    p {
                        "your vote: "
                        @if item.vote == 1 {
                            "Yes"
                        } @else if item.vote == -1 {
                            "No"
                        }
                    }
                }
                p { "yes: " (item.yes_count) ", no: " (item.no_count) }
                p { (human_relative_time(item.author_timestamp)) }
                input type="hidden" value=(item.statement_id) {}
            }
        }
    };
    Ok(base(cookies, None, &maybe_user, content))
}
