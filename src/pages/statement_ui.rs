use crate::{
    highlight::highlight_html,
    pages::charts::yes_no_pie_chart,
    structs::{Statement, User, Vote},
};

use anyhow::Result;
use maud::{html, Escaper, Markup, PreEscaped};
use sqlx::SqlitePool;

use crate::util::human_relative_time;
use std::fmt::Write;

pub async fn small_statement_content(
    statement: &Statement,
    timestamp: Option<i64>,
    show_controls: bool,
    maybe_user: &Option<User>,
    pool: &SqlitePool,
) -> Result<Markup> {
    let mut statement_text_html_escaped = String::new();
    write!(
        Escaper::new(&mut statement_text_html_escaped),
        "{}",
        statement.text
    )?;
    let statement_highlighted = highlight_html(&statement_text_html_escaped);
    Ok(html! {
        div class="flex flex-col w-full p-4" {
            @if let Some(timestamp) = timestamp {
                div class="opacity-50" {
                    (human_relative_time(timestamp))
                }
            }
            div class="h-full" {
                a href=(format!("/statement/{}", statement.id)) {
                    span class="text-lg" data-testid="statement-text" { (PreEscaped(statement_highlighted)) }
                }
            }
            @if show_controls {
                div class="mt-4 flex items-center gap-4" {
                    a href=(format!("/new?target={}&target_all=true", statement.id)) {
                        "add follow-up"
                    }
                    (subscribe_button(statement.id, maybe_user, pool).await?)
                }
            }
        }
    })
}

pub async fn small_statement_piechart(statement_id: i64, pool: &SqlitePool) -> Result<Markup> {
    Ok(html! {
        div class="py-2" {
            (yes_no_pie_chart(statement_id, pool).await?)
        }
    })
}

pub fn small_statement_vote(vote: Option<Vote>) -> Result<Markup> {
    let vote_color = match vote {
        Some(Vote::Yes) => "bg-green-600",
        Some(Vote::No) => "bg-red-600",
        Some(Vote::Skip) => "",
        None => "",
    };
    Ok(html! {
        div class={"font-bold text-white w-16 py-2 flex items-center justify-center shrink-0 rounded-r-lg "(vote_color)} {
            @match vote {
                Some(Vote::Yes) => "YES",
                Some(Vote::No) => "NO",
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
            span class="opacity-50" { "subscribed" }
        } @else {
            form hx-post="/subscribe" {
                input type="hidden" name="statement_id" value=(statement_id);
                button { "subscribe" }
            }
        }
    })
}
