use super::base::base;
use crate::{
    db::{get_followups, get_statement, statement_stats},
    error::Error,
    pages::statement_ui::{
        small_statement_content, small_statement_piechart, small_statement_vote,
        small_statement_vote_fetch,
    },
    structs::{Statement, StatementStats, User, Vote},
};

use axum::{extract::Path, Extension};
use maud::{html, Markup};
use sqlx::SqlitePool;
use tower_cookies::Cookies;

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
    let statement: Option<Statement> = get_statement(statement_id, &pool).await.ok();
    let user_vote = match &maybe_user {
        Some(user) => user.get_vote(statement_id, &pool).await?,
        None => None,
    };
    let content = html! {
        @if let Some(statement) = statement {
            div.shadow style="display:flex; border-radius: 10px" {
                div style="width: 100%; font-size: 1.5em; padding:1em;" {
                    (statement.text)
                }
                @if user_vote.is_some() {
                    (small_statement_piechart(statement.id, &pool).await?)
                    (small_statement_vote(user_vote)?)
                }
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
            @match user_vote {
                Some(_) => {
                    h2 { "Follow-ups" }
                    @let followups = get_followups(statement_id, &pool).await?;
                    @if followups.is_empty() {
                        div { "No follow-ups yet." }
                    }
                    @for statement_id in followups {
                        // TODO: different columns depending on vote-dependent follow up
                        div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                            (small_statement_content(&get_statement(statement_id, &pool).await?, None, &maybe_user, &pool).await?)
                            (small_statement_piechart(statement_id, &pool).await?)
                            (small_statement_vote_fetch(statement_id, &maybe_user, &pool).await?)
                        }
                    }
                }
                None => (history(&maybe_user, &pool).await?)
            }
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
            @let statement = Statement {
                id: item.statement_id,
                text: item.statement_text,
            };
            div.shadow style="display:flex; margin-bottom: 20px; border-radius: 10px;" {
                (small_statement_content(&statement, Some(item.vote_timestamp), &maybe_user, &pool).await?)
                (small_statement_piechart(item.statement_id, &pool).await?)
                (small_statement_vote(Some(Vote::from(item.vote)?))?)
            }
        }
    })
}
