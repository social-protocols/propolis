use super::base::base;
use crate::{
    db::{get_statement, statement_stats},
    error::Error,
    pages::charts::yes_no_pie_chart,
    structs::{StatementStats, User},
};

use axum::{extract::Path, Extension};
use maud::{html, Markup};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

use crate::util::human_relative_time;

/// Returns an apexchart div with votes of the particular statement
pub async fn votes(
    Path(statement_id): Path<i64>,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Markup, Error> {
    let StatementStats {
        yes_votes,
        skip_votes,
        no_votes,
        ..
    } = statement_stats(statement_id, &pool).await?;
    Ok(html! {
        div id="chart" {}
        script type="text/javascript" {
            (format!("setupChart('#chart', {yes_votes},{skip_votes},{no_votes});"))
        }
    })
}

pub async fn statement_page(
    Path(statement_id): Path<i64>,
    maybe_user: Option<User>,
    cookies: Cookies,
    Extension(pool): Extension<SqlitePool>,
) -> Result<Markup, Error> {
    let statement = get_statement(statement_id, &pool).await?;
    let content = html! {
        @if let Some(statement) = statement {
            div.shadow style="font-size: 1.5em; padding: 1em; border-radius: 10px" {
                (statement.text)
            }
            div.row style="margin-bottom: 50px" {
                div.col {
                    form form id="form" hx-post="/vote" {
                        input type="hidden" value=(statement_id) name="statement_id";
                        button name="vote" value="Yes" { "Agree" }
                        button name="vote" value="Skip" { "Skip" }
                        button name="vote" value="ItDepends" { "It depends" }
                        button name="vote" value="No" { "Disagree" }
                    }
                }
            }
            (history(&maybe_user, &pool).await?)
        } @else {
            div { "Statement not found." }
        }
    };

    Ok(base(cookies, None, &maybe_user, content))
}

async fn history(maybe_user: &Option<User>, pool: &SqlitePool) -> Result<Markup, Error> {
    let history = match maybe_user {
        Some(user) => user.vote_history(&pool).await?,
        None => Vec::new(),
    };

    Ok(html! {
        @for item in history {
            div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                div style="width: 100%; padding: 15px" {
                    div style="opacity: 0.5" {
                        (human_relative_time(&item.vote_timestamp))
                    }
                    div {
                        a href=(format!("/statement/{}", item.statement_id)) style="text-decoration: none"  {
                            span style="color: var(--cfg);" { (item.statement_text) }
                        }
                    }
                    div style="display: flex; gap: 12px" {
                        a href=(format!("/new?target={}", item.statement_id)) {
                            "↰ Reply"
                        }
                        (follow_button(item.statement_id, &maybe_user, &pool).await?)
                    }
                }
                div style="padding: 5px 0px; align-self: center;" {
                    (yes_no_pie_chart(item.statement_id, &pool).await?)
                }
                @let vote_color = if item.vote == 1 {
                    "forestgreen"
                } else if item.vote == -1 {
                    "firebrick"
                } else {
                    "default"
                };
                div style={"font-weight:bold; background-color: "(vote_color)"; color: white; width: 60px; display: flex; align-items:center; justify-content: center; border-top-right-radius: 10px; border-bottom-right-radius: 10px;"} {
                    span {
                        @if item.vote == 1 {
                            "YES"
                        } @else if item.vote == -1 {
                            "NO"
                        }
                    }
                }
            }
        }
    })
}

pub async fn follow_button(
    statement_id: i64,
    maybe_user: &Option<User>,
    pool: &SqlitePool,
) -> Result<Markup, Error> {
    let is_following = match maybe_user {
        Some(user) => user.is_following(statement_id, pool).await?,
        None => false,
    };

    Ok(html! {
        @if is_following {
            "following"
        } @else {
            form hx-post="/follow" {
                input type="hidden" name="statement_id" value=(statement_id);
                button { "Follow" }
            }
        }
    })
}
