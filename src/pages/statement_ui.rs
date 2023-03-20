use crate::{
    error::Error,
    pages::charts::yes_no_pie_chart,
    structs::{Statement, User, Vote},
};

use maud::{html, Markup};
use sqlx::SqlitePool;

use crate::util::human_relative_time;

pub async fn small_statement_content(
    statement: &Statement,
    timestamp: Option<i64>,
    maybe_user: &Option<User>,
    pool: &SqlitePool,
) -> Result<Markup, Error> {
    Ok(html! {
        div style="width: 100%; padding: 15px" {
            @if let Some(timestamp) = timestamp {
                div style="opacity: 0.5" {
                    (human_relative_time(timestamp))
                }
            }
            div {
                a href=(format!("/statement/{}", statement.id)) style="text-decoration: none"  {
                    span style="color: var(--cfg);" { (statement.text) }
                }
            }
            div style="display: flex; gap: 12px" {
                a href=(format!("/new?target={}", statement.id)) {
                    "↰ Reply"
                }
                (subscribe_button(statement.id, &maybe_user, &pool).await?)
            }
        }
    })
}

pub async fn small_statement_piechart(
    statement_id: i64,
    pool: &SqlitePool,
) -> Result<Markup, Error> {
    Ok(html! {
        div style="padding: 5px 0px; align-self: center;" {
            (yes_no_pie_chart(statement_id, &pool).await?)
        }
    })
}

pub fn small_statement_vote(vote: Vote) -> Result<Markup, Error> {
    let vote_color = if vote == Vote::Yes {
        "forestgreen"
    } else if vote == Vote::No {
        "firebrick"
    } else {
        "default"
    };
    Ok(html! {
        div style={"font-weight:bold; background-color: "(vote_color)"; color: white; width: 60px; display: flex; align-items:center; justify-content: center; border-top-right-radius: 10px; border-bottom-right-radius: 10px;"} {
            span {
                @if vote == Vote::Yes {
                    "YES"
                } @else if vote == Vote::No {
                    "NO"
                }
            }
        }
    })
}

pub async fn subscribe_button(
    statement_id: i64,
    maybe_user: &Option<User>,
    pool: &SqlitePool,
) -> Result<Markup, Error> {
    let is_subscribed = match maybe_user {
        Some(user) => user.is_subscribed(statement_id, pool).await?,
        None => false,
    };

    Ok(html! {
        @if is_subscribed {
            "subscribed"
        } @else {
            form hx-post="/subscribe" {
                input type="hidden" name="statement_id" value=(statement_id);
                button { "Subscribe" }
            }
        }
    })
}
