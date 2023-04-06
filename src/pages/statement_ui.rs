use crate::{
    pages::charts::yes_no_pie_chart,
    structs::{Statement, User, Vote},
};

use anyhow::Result;
use maud::{html, Markup};
use sqlx::SqlitePool;

use crate::util::human_relative_time;

pub async fn small_statement_content(
    statement: &Statement,
    timestamp: Option<i64>,
    show_controls: bool,
    maybe_user: &Option<User>,
    pool: &SqlitePool,
) -> Result<Markup> {
    Ok(html! {
        div style="display:flex; flex-direction: column; width: 100%; padding: 15px" {
            @if let Some(timestamp) = timestamp {
                div style="opacity: 0.5" {
                    (human_relative_time(timestamp))
                }
            }
            div style="height:100%" {
                a href=(format!("/statement/{}", statement.id)) style="text-decoration: none"  {
                    span style="color: var(--cfg);" { (statement.text) }
                }
            }
            @if show_controls {
                div style="display: flex; align-items:center; gap: 12px" {
                    // link with button styles
                    a style="text-decoration: none; font-weight: bold; letter-spacing: 0.1em; font-size: 85%; padding: 0.4em 0em; margin-right: 1em" href=(format!("/new?target={}", statement.id)) {
                        "↳ Add Follow-Up"
                    }
                    (subscribe_button(statement.id, maybe_user, pool).await?)
                }
            }
        }
    })
}

pub async fn small_statement_piechart(statement_id: i64, pool: &SqlitePool) -> Result<Markup> {
    Ok(html! {
        div style="padding: 5px 0px; align-self: center;" {
            (yes_no_pie_chart(statement_id, pool).await?)
        }
    })
}

pub fn small_statement_vote(vote: Option<Vote>) -> Result<Markup> {
    let vote_color = match vote {
        Some(Vote::Yes) => "forestgreen",
        Some(Vote::No) => "firebrick",
        Some(Vote::ItDepends) => "slategrey",
        Some(Vote::Skip) => "default",
        None => "default",
    };
    Ok(html! {
        div style={"font-weight:bold; background-color: "(vote_color)"; color: white; width: 60px; display: flex; align-items:center; justify-content: center; border-top-right-radius: 10px; border-bottom-right-radius: 10px; flex-shrink: 0"} {
            @match vote {
                Some(Vote::Yes) => "YES",
                Some(Vote::No) => "NO",
                Some(Vote::ItDepends) => span style="writing-mode: tb-rl" { "IT DEPENDS" },
                Some(Vote::Skip) => "SKIP",
                None => "",
            }
        }
    })
}

pub async fn small_statement_vote_fetch(
    statement_id: i64,
    maybe_user: &Option<User>,
    pool: &SqlitePool,
) -> Result<Markup> {
    let vote = match maybe_user {
        Some(user) => user.get_vote(statement_id, pool).await?,
        None => None,
    };
    small_statement_vote(vote)
}

#[cfg(not(feature = "with_predictions"))]
pub async fn small_statement_predictions(
    _statement: &Statement,
    _pool: &SqlitePool,
) -> Result<Markup> {
    Ok(html! {})
}

#[cfg(feature = "with_predictions")]
pub async fn small_statement_predictions(
    statement: &Statement,
    pool: &SqlitePool,
) -> Result<Markup> {
    let btn_style = |clr| {
        format!("color: white; background-color: {clr}; border-color: forestgreen; padding: 4px")
    };
    Ok(html! {
        div style={"float: left; font-size: 0.8em; align-self: center"} {
            @match statement.get_meta(pool).await? {
                Some(crate::prediction::prompts::StatementMeta::Politics{tags, ideologies: _}) => {
                    @for tag in &tags {
                        div {
                            button style=(btn_style("#088")) { (tag.value) }
                        }
                    }
                }
                Some(crate::prediction::prompts::StatementMeta::Personal{tags, bfp_traits: _}) => {
                    @for tag in &tags {
                        div {
                            button style=(btn_style("#0b9")) { (tag.value) }
                        }
                    }
                }
                _ => {}
            }
        }
    })
}

pub async fn subscribe_button(
    statement_id: i64,
    maybe_user: &Option<User>,
    pool: &SqlitePool,
) -> Result<Markup> {
    let is_subscribed = match maybe_user {
        Some(user) => user.is_subscribed(statement_id, pool).await?,
        None => false,
    };

    Ok(html! {
        @if is_subscribed {
            // same padding as button
            span style="padding: 0.4em 1em; opacity: 0.5; font-size: 85%" { "subscribed" }
        } @else {
            form hx-post="/subscribe" {
                input type="hidden" name="statement_id" value=(statement_id);
                button style="border: none; background: none; margin: 0;" { "Subscribe" }
            }
        }
    })
}
